import * as anchor from "@project-serum/anchor";
import { AnchorError, Program } from "@project-serum/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  createAssociatedTokenAccount,
  createMint,
  getMint,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { SerumGov } from "../target/types/serum_gov";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { assert, expect } from "chai";
import { BN } from "bn.js";

describe("serum-gov", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const { connection } = provider;
  const program = anchor.workspace.SerumGov as Program<SerumGov>;

  let SRM_MINT: PublicKey;
  let MSRM_MINT: PublicKey;

  let srmVault: PublicKey;
  let msrmVault: PublicKey;

  // const fileBuffer = fs.readFileSync("./keys/upgrade.json");
  // const secretKey: number[] = JSON.parse(fileBuffer.toString());
  // const sbf = Keypair.fromSecretKey(Uint8Array.from(secretKey));

  const sbf = Keypair.generate();

  const alice = Keypair.generate();

  let aliceSRMAccount: PublicKey;
  let aliceMSRMAccount: PublicKey;
  let aliceGSRMAccount: PublicKey;

  const [authority, authorityBump] = findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  );
  const [GSRM_MINT] = findProgramAddressSync(
    [Buffer.from("gSRM")],
    program.programId
  );
  const [configAccount] = findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  const [aliceUserAccount] = findProgramAddressSync(
    [Buffer.from("user"), alice.publicKey.toBuffer()],
    program.programId
  );

  before(async () => {
    // Airdrop sbf
    const sbfDrop = await connection.requestAirdrop(
      sbf.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sbfDrop);

    // Airdrop alice
    const aliceDrop = await connection.requestAirdrop(
      alice.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(aliceDrop);

    // Create SRM mint
    SRM_MINT = await createMint(
      connection,
      sbf,
      sbf.publicKey,
      sbf.publicKey,
      6,
      undefined,
      {
        commitment: "confirmed",
      }
    );

    // Create MSRM mint
    MSRM_MINT = await createMint(
      connection,
      sbf,
      sbf.publicKey,
      sbf.publicKey,
      0,
      undefined,
      {
        commitment: "confirmed",
      }
    );

    // Create alice SRM account
    aliceSRMAccount = await createAssociatedTokenAccount(
      connection,
      alice,
      SRM_MINT,
      alice.publicKey
    );

    // Create alice MSRM account
    aliceMSRMAccount = await createAssociatedTokenAccount(
      connection,
      alice,
      MSRM_MINT,
      alice.publicKey
    );

    // Mint SRM to alice
    await mintTo(
      connection,
      sbf,
      SRM_MINT,
      aliceSRMAccount,
      sbf,
      BigInt(200 * 1000000)
    );

    // Mint MSRM to alice
    await mintTo(connection, sbf, MSRM_MINT, aliceMSRMAccount, sbf, 2);

    [srmVault] = findProgramAddressSync(
      [Buffer.from("vault"), SRM_MINT.toBuffer()],
      program.programId
    );
    [msrmVault] = findProgramAddressSync(
      [Buffer.from("vault"), MSRM_MINT.toBuffer()],
      program.programId
    );
  });

  it("can init", async () => {
    await program.methods
      .init(new BN(0), new BN(1000))
      .accounts({
        payer: sbf.publicKey,
        authority,
        config: configAccount,
        gsrmMint: GSRM_MINT,
        srmMint: SRM_MINT,
        srmVault,
        msrmMint: MSRM_MINT,
        msrmVault,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([sbf])
      .rpc();

    const config = await program.account.config.fetch(configAccount);
    expect(config.claimDelay.toNumber()).to.equal(0);
    expect(config.redeemDelay.toNumber()).to.equal(1000);

    const mint = await getMint(connection, GSRM_MINT);
    expect(mint.decimals).to.equal(6);
    expect(mint.mintAuthority.toBase58()).to.equal(authority.toBase58());

    const vaultSrm = await connection.getTokenAccountBalance(srmVault);
    expect(vaultSrm.value.uiAmount).to.equal(0);

    const vaultMsrm = await connection.getTokenAccountBalance(msrmVault);
    expect(vaultMsrm.value.uiAmount).to.equal(0);
  });

  it("cant init vaults twice", async () => {
    try {
      await program.methods
        .init(new BN(0), new BN(1000))
        .accounts({
          payer: sbf.publicKey,
          authority,
          config: configAccount,
          gsrmMint: GSRM_MINT,
          srmMint: SRM_MINT,
          srmVault,
          msrmMint: MSRM_MINT,
          msrmVault,
          rent: SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([sbf])
        .rpc();
      assert(false);
    } catch (e) {
      assert(true);
    }
  });

  it("can init user", async () => {
    const tx = await program.methods
      .initUser()
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceAccount = await program.account.user.fetch(aliceUserAccount);

    expect(aliceAccount.owner.toBase58()).to.equal(alice.publicKey.toBase58());
    expect(aliceAccount.claimIndex.toNumber()).to.equal(0);
    expect(aliceAccount.redeemIndex.toNumber()).to.equal(0);
  });

  it("can deposit srm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [claimTicket] = findProgramAddressSync(
      [
        Buffer.from("claim"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.claimIndex.toString()),
      ],
      program.programId
    );

    await program.methods
      .depositSrm(new BN(100_000_000))
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        srmMint: SRM_MINT,
        ownerSrmAccount: aliceSRMAccount,
        authority,
        config: configAccount,
        srmVault,
        claimTicket,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceClaimTicket = await program.account.claimTicket.fetch(
      claimTicket
    );
    const srmVaultBalance = await connection.getTokenAccountBalance(srmVault);

    expect(aliceClaimTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceClaimTicket.claimIndex.toNumber()).to.equal(0);
    expect(aliceClaimTicket.amount.toNumber()).to.equal(100_000_000);
    expect(aliceClaimTicket.isMsrm).to.equal(false);

    expect(srmVaultBalance.value.uiAmount).to.equal(100);
  });

  it("can deposit msrm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [msrmTicket] = findProgramAddressSync(
      [
        Buffer.from("claim"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.claimIndex.toString()),
      ],
      program.programId
    );

    await program.methods
      .depositMsrm(new BN(1))
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        msrmMint: MSRM_MINT,
        config: configAccount,
        ownerMsrmAccount: aliceMSRMAccount,
        authority,
        msrmVault,
        claimTicket: msrmTicket,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc({ skipPreflight: true });

    const aliceClaimTicket = await program.account.claimTicket.fetch(
      msrmTicket
    );
    const msrmVaultBalance = await connection.getTokenAccountBalance(msrmVault);

    expect(aliceClaimTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceClaimTicket.claimIndex.toNumber()).to.equal(1);
    expect(aliceClaimTicket.amount.toNumber()).to.equal(1);
    expect(aliceClaimTicket.isMsrm).to.equal(true);

    expect(msrmVaultBalance.value.uiAmount).to.equal(1);
  });

  it("can claim for srm locker", async () => {
    // const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    aliceGSRMAccount = await createAssociatedTokenAccount(
      connection,
      alice,
      GSRM_MINT,
      alice.publicKey
    );

    const [aliceSrmTicket] = findProgramAddressSync(
      [Buffer.from("claim"), alice.publicKey.toBuffer(), Buffer.from("0")],
      program.programId
    );

    await program.methods
      .claim(new BN(0))
      .accounts({
        owner: alice.publicKey,
        ticket: aliceSrmTicket,
        config: configAccount,
        authority,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceGsrmBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount
    );

    expect(aliceGsrmBalance.value.uiAmount).to.equal(100);

    try {
      await program.account.claimTicket.fetch(aliceSrmTicket);
      assert(false);
    } catch (e) {
      if (e instanceof Error) {
        assert(true);
      }
    }
  });

  it("can claim for msrm locker", async () => {
    const [aliceMsrmTicket] = findProgramAddressSync(
      [Buffer.from("claim"), alice.publicKey.toBuffer(), Buffer.from("1")],
      program.programId
    );

    await program.methods
      .claim(new BN(1))
      .accounts({
        owner: alice.publicKey,
        ticket: aliceMsrmTicket,
        config: configAccount,
        authority,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceGsrmBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount
    );

    expect(aliceGsrmBalance.value.uiAmount).to.equal(100 + 1_000_000);

    try {
      await program.account.claimTicket.fetch(aliceMsrmTicket);
      assert(false);
    } catch (e) {
      if (e instanceof Error) {
        assert(true);
      }
    }
  });

  it("cant claim locker twice", async () => {
    const [aliceSrmTicket] = findProgramAddressSync(
      [Buffer.from("claim"), alice.publicKey.toBuffer(), Buffer.from("0")],
      program.programId
    );

    try {
      await program.methods
        .claim(new BN(0))
        .accounts({
          owner: alice.publicKey,
          ticket: aliceSrmTicket,
          config: configAccount,
          authority,
          gsrmMint: GSRM_MINT,
          ownerGsrmAccount: aliceGSRMAccount,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice])
        .rpc();
      assert(false);
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else {
        console.error(e);
        assert(false);
      }
    }
  });

  it("can update config", async () => {
    await program.methods
      .updateConfig(new BN(2000), new BN(2000))
      .accounts({
        upgradeAuthority: sbf.publicKey,
        config: configAccount,
      })
      .signers([sbf])
      .rpc();

    const config = await program.account.config.fetch(configAccount);
    expect(config.claimDelay.toNumber()).to.equal(2000);
    expect(config.redeemDelay.toNumber()).to.equal(2000);
  });

  // it("cant claim locker before delay", async () => {
  //   const aliceAccount = await program.account.user.fetch(aliceUserAccount);
  //   const [locker] = findProgramAddressSync(
  //     [
  //       Buffer.from("locker"),
  //       alice.publicKey.toBuffer(),
  //       Buffer.from(aliceAccount.lockerIndex.toString()),
  //     ],
  //     program.programId
  //   );

  //   await program.methods
  //     .depositSrm(new BN(100_000_000), new BN(1_000), new BN(1_000))
  //     .accounts({
  //       owner: alice.publicKey,
  //       userAccount: aliceUserAccount,
  //       srmMint: SRM_MINT,
  //       ownerSrmAccount: aliceSRMAccount,
  //       authority,
  //       srmVault,
  //       locker,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice])
  //     .rpc();

  //   try {
  //     await program.methods
  //       .claim(aliceAccount.lockerIndex)
  //       .accounts({
  //         owner: alice.publicKey,
  //         locker: locker,
  //         authority,
  //         gsrmMint: GSRM_MINT,
  //         ownerGsrmAccount: aliceGSRMAccount,
  //         clock: SYSVAR_CLOCK_PUBKEY,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([alice])
  //       .rpc();
  //     assert(false);
  //   } catch (e) {
  //     if (e instanceof AnchorError) {
  //       assert(true, e.error.errorMessage);
  //     } else {
  //       console.error(e);
  //       assert(false);
  //     }
  //   }
  // });
});
