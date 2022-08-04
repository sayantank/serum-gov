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

const MSRM_MULTIPLIER = 1_000_000_000_000;

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
      .init(new BN(0), new BN(0))
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
    expect(config.redeemDelay.toNumber()).to.equal(0);

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
    // CLAIM_INDEX = 0
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
      .depositSrm(new BN(200_000_000))
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
    expect(aliceClaimTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceClaimTicket.claimIndex.toNumber()).to.equal(0);
    expect(aliceClaimTicket.amount.toNumber()).to.equal(200_000_000);
    expect(aliceClaimTicket.isMsrm).to.equal(false);

    const srmVaultBalance = await connection.getTokenAccountBalance(srmVault);
    expect(srmVaultBalance.value.uiAmount).to.equal(200);
  });

  it("can deposit msrm", async () => {
    // CLAIM_INDEX = 1
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
    expect(aliceClaimTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceClaimTicket.claimIndex.toNumber()).to.equal(1);
    expect(aliceClaimTicket.amount.toNumber()).to.equal(1);
    expect(aliceClaimTicket.isMsrm).to.equal(true);

    const msrmVaultBalance = await connection.getTokenAccountBalance(msrmVault);
    expect(msrmVaultBalance.value.uiAmount).to.equal(1);
  });

  it("can claim for srm ticket", async () => {
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

    expect(aliceGsrmBalance.value.uiAmount).to.equal(200);

    try {
      await program.account.claimTicket.fetch(aliceSrmTicket);
      assert(false);
    } catch (e) {
      if (e instanceof Error) {
        assert(true);
      }
    }
  });

  it("can claim for msrm ticket", async () => {
    const [aliceMsrmTicket] = findProgramAddressSync(
      [Buffer.from("claim"), alice.publicKey.toBuffer(), Buffer.from("1")],
      program.programId
    );

    await program.methods
      .claim(new BN(1))
      .accounts({
        owner: alice.publicKey,
        ticket: aliceMsrmTicket,
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

    expect(aliceGsrmBalance.value.uiAmount).to.equal(200 + 1_000_000);

    try {
      await program.account.claimTicket.fetch(aliceMsrmTicket);
      assert(false);
    } catch (e) {
      if (e instanceof Error) {
        assert(true);
      }
    }
  });

  it("can burn gsrm for srm", async () => {
    // REDEEM_INDEX = 0
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [redeemTicket] = findProgramAddressSync(
      [
        Buffer.from("redeem"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.redeemIndex.toString()),
      ],
      program.programId
    );

    await program.methods
      .burnGsrm(new BN(100_000_000), false)
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        authority,
        config: configAccount,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount,
        redeemTicket,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceRedeemTicket = await program.account.redeemTicket.fetch(
      redeemTicket
    );
    expect(aliceRedeemTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceRedeemTicket.isMsrm).to.equal(false);
    expect(aliceRedeemTicket.redeemDelay.toNumber()).to.equal(0);
    expect(aliceRedeemTicket.amount.toNumber()).to.equal(100_000_000);

    const aliceGSRMBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount
    );
    expect(aliceGSRMBalance.value.uiAmount).to.equal(1_000_100);
  });

  it("can burn gsrm for msrm", async () => {
    // REDEEM_INDEX = 1
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [redeemTicket] = findProgramAddressSync(
      [
        Buffer.from("redeem"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.redeemIndex.toString()),
      ],
      program.programId
    );

    await program.methods
      .burnGsrm(new BN(MSRM_MULTIPLIER), true)
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        authority,
        config: configAccount,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount,
        redeemTicket,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceRedeemTicket = await program.account.redeemTicket.fetch(
      redeemTicket
    );
    expect(aliceRedeemTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceRedeemTicket.isMsrm).to.equal(true);
    expect(aliceRedeemTicket.redeemDelay.toNumber()).to.equal(0);
    expect(aliceRedeemTicket.amount.toNumber()).to.equal(1);

    const aliceGSRMBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount
    );
    expect(aliceGSRMBalance.value.uiAmount).to.equal(100);
  });

  it("cant burn gsrm for msrm with invalid amount", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [redeemTicket] = findProgramAddressSync(
      [
        Buffer.from("redeem"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.redeemIndex.toString()),
      ],
      program.programId
    );

    try {
      await program.methods
        .burnGsrm(new BN(MSRM_MULTIPLIER + 10), true)
        .accounts({
          owner: alice.publicKey,
          userAccount: aliceUserAccount,
          authority,
          config: configAccount,
          gsrmMint: GSRM_MINT,
          ownerGsrmAccount: aliceGSRMAccount,
          redeemTicket,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice])
        .rpc();
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else {
        console.error(e);
        assert(false);
      }
    }
  });

  it("cant redeem msrm with srm ticket", async () => {
    const [redeemTicket] = findProgramAddressSync(
      [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("0")],
      program.programId
    );

    try {
      await program.methods
        .redeemMsrm(new BN(1))
        .accounts({
          owner: alice.publicKey,
          authority,
          redeemTicket,
          msrmMint: MSRM_MINT,
          msrmVault,
          ownerMsrmAccount: aliceMSRMAccount,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice])
        .rpc();
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else {
        console.error(e);
        assert(false);
      }
    }
  });

  it("can redeem srm", async () => {
    const [redeemTicket] = findProgramAddressSync(
      [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("0")],
      program.programId
    );

    await program.methods
      .redeemSrm(new BN(0))
      .accounts({
        owner: alice.publicKey,
        authority,
        redeemTicket,
        srmMint: SRM_MINT,
        srmVault,
        ownerSrmAccount: aliceSRMAccount,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceSrmBalance = await connection.getTokenAccountBalance(
      aliceSRMAccount
    );
    expect(aliceSrmBalance.value.uiAmount).to.equal(100);

    try {
      await program.account.redeemTicket.fetch(redeemTicket);
      assert(false);
    } catch (e) {
      if (e instanceof Error) {
        assert(true);
      }
    }
  });

  it("cant redeem srm with msrm ticket", async () => {
    const [redeemTicket] = findProgramAddressSync(
      [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("1")],
      program.programId
    );

    try {
      await program.methods
        .redeemSrm(new BN(1))
        .accounts({
          owner: alice.publicKey,
          authority,
          redeemTicket,
          srmMint: SRM_MINT,
          srmVault,
          ownerSrmAccount: aliceSRMAccount,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice])
        .rpc();
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else {
        console.error(e);
        assert(false);
      }
    }
  });

  it("can redeem msrm", async () => {
    const [redeemTicket] = findProgramAddressSync(
      [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("1")],
      program.programId
    );

    await program.methods
      .redeemMsrm(new BN(1))
      .accounts({
        owner: alice.publicKey,
        authority,
        redeemTicket,
        msrmMint: MSRM_MINT,
        msrmVault,
        ownerMsrmAccount: aliceMSRMAccount,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceMsrmBalance = await connection.getTokenAccountBalance(
      aliceMSRMAccount
    );
    expect(aliceMsrmBalance.value.uiAmount).to.equal(2);

    try {
      await program.account.redeemTicket.fetch(redeemTicket);
      assert(false);
    } catch (e) {
      if (e instanceof Error) {
        assert(true);
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

  it("cant claim before claim_delay", async () => {
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

    try {
      await program.methods
        .claim(aliceAccount.claimIndex)
        .accounts({
          owner: alice.publicKey,
          ticket: claimTicket,
          authority,
          gsrmMint: GSRM_MINT,
          ownerGsrmAccount: aliceGSRMAccount,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice])
        .rpc();
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else {
        console.error(e);
        assert(false);
      }
    }
  });

  it("cant redeem before redeem_delay", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [redeemTicket] = findProgramAddressSync(
      [
        Buffer.from("redeem"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.redeemIndex.toString()),
      ],
      program.programId
    );

    await program.methods
      .burnGsrm(new BN(100_000_000), false)
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        authority,
        config: configAccount,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount,
        redeemTicket,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    try {
      await program.methods
        .redeemSrm(aliceAccount.redeemIndex)
        .accounts({
          owner: alice.publicKey,
          authority,
          redeemTicket,
          srmMint: SRM_MINT,
          srmVault,
          ownerSrmAccount: aliceSRMAccount,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice])
        .rpc();
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else {
        console.error(e);
        assert(false);
      }
    }
  });
});
