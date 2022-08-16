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
  createAccount,
  createAssociatedTokenAccount,
  createMint,
  getAssociatedTokenAddress,
  getMint,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { SerumGov } from "../target/types/serum_gov";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { assert, expect } from "chai";
import { BN } from "bn.js";
import { sleep } from "./utils";

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

  let sbfSrmAccount: PublicKey;

  let aliceSRMAccount = Keypair.generate();
  let aliceMSRMAccount = Keypair.generate();
  let aliceGSRMAccount = Keypair.generate();

  const [authority] = findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  );
  const [GSRM_MINT] = findProgramAddressSync(
    [Buffer.from("gSRM")],
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
    await createAccount(
      connection,
      alice,
      SRM_MINT,
      alice.publicKey,
      aliceSRMAccount
    );

    // Create alice MSRM account
    await createAccount(
      connection,
      alice,
      MSRM_MINT,
      alice.publicKey,
      aliceMSRMAccount
    );

    // Create sbf SRM account
    await createAccount(connection, sbf, SRM_MINT, sbf.publicKey);

    // Mint SRM to alice
    await mintTo(
      connection,
      sbf,
      SRM_MINT,
      aliceSRMAccount.publicKey,
      sbf,
      BigInt(50000 * 1000000)
    );

    // Mint MSRM to alice
    await mintTo(
      connection,
      sbf,
      MSRM_MINT,
      aliceMSRMAccount.publicKey,
      sbf,
      2
    );

    sbfSrmAccount = await getAssociatedTokenAddress(
      SRM_MINT,
      sbf.publicKey,
      true
    );
    await mintTo(
      connection,
      sbf,
      SRM_MINT,
      sbfSrmAccount,
      sbf,
      BigInt(50000 * 1000000)
    );

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
      .init()
      .accounts({
        payer: sbf.publicKey,
        authority,
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

    const mint = await getMint(connection, GSRM_MINT);
    expect(mint.decimals).to.equal(6);
    expect(mint.mintAuthority.toBase58()).to.equal(authority.toBase58());

    const vaultSrm = await connection.getTokenAccountBalance(srmVault);
    expect(vaultSrm.value.uiAmount).to.equal(0);

    const vaultMsrm = await connection.getTokenAccountBalance(msrmVault);
    expect(vaultMsrm.value.uiAmount).to.equal(0);

    await createAccount(
      connection,
      alice,
      GSRM_MINT,
      alice.publicKey,
      aliceGSRMAccount
    );
  });

  it("cant init twice", async () => {
    try {
      await program.methods
        .init()
        .accounts({
          payer: sbf.publicKey,
          authority,
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
      .initUser(alice.publicKey)
      .accounts({
        payer: alice.publicKey,
        userAccount: aliceUserAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceAccount = await program.account.user.fetch(aliceUserAccount);

    expect(aliceAccount.owner.toBase58()).to.equal(alice.publicKey.toBase58());
    expect(aliceAccount.lockIndex.toNumber()).to.equal(0);
    expect(aliceAccount.vestIndex.toNumber()).to.equal(0);
  });

  it("can deposit srm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [aliceLockedAccount] = findProgramAddressSync(
      [
        Buffer.from("locked_account"),
        alice.publicKey.toBuffer(),
        aliceAccount.lockIndex.toBuffer("le", 8),
      ],
      program.programId
    );

    const claimTicket = Keypair.generate();

    await program.methods
      .depositLockedSrm(new BN(200_000_000))
      .accounts({
        payer: sbf.publicKey,
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        srmMint: SRM_MINT,
        payerSrmAccount: sbfSrmAccount,
        authority,
        srmVault,
        lockedAccount: aliceLockedAccount,
        claimTicket: claimTicket.publicKey,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([sbf, claimTicket])
      .rpc();

    const aliceClaimTicket = await program.account.claimTicket.fetch(
      claimTicket.publicKey
    );
    expect(aliceClaimTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceClaimTicket.gsrmAmount.toNumber()).to.equal(200_000_000);
    expect(aliceClaimTicket.claimDelay.toNumber()).to.equal(2);

    const srmVaultBalance = await connection.getTokenAccountBalance(srmVault);
    expect(srmVaultBalance.value.uiAmount).to.equal(200);
  });

  it("can deposit msrm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [aliceLockedAccount] = findProgramAddressSync(
      [
        Buffer.from("locked_account"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.lockIndex.toBuffer("le", 8)),
      ],
      program.programId
    );

    let claimTicket = Keypair.generate();

    await program.methods
      .depositLockedMsrm(new BN(1))
      .accounts({
        payer: alice.publicKey,
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        msrmMint: MSRM_MINT,
        payerMsrmAccount: aliceMSRMAccount.publicKey,
        authority,
        msrmVault,
        lockedAccount: aliceLockedAccount,
        claimTicket: claimTicket.publicKey,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice, claimTicket])
      .rpc();

    const aliceClaimTicket = await program.account.claimTicket.fetch(
      claimTicket.publicKey
    );
    expect(aliceClaimTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceClaimTicket.gsrmAmount.toNumber()).to.equal(
      1 * MSRM_MULTIPLIER
    );
    expect(aliceClaimTicket.claimDelay.toNumber()).to.equal(2);

    const msrmVaultBalance = await connection.getTokenAccountBalance(msrmVault);
    expect(msrmVaultBalance.value.uiAmount).to.equal(1);
  });

  it("can claim tickets", async () => {
    // const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    await sleep(3);

    let aliceClaimTickets = await program.account.claimTicket.all([
      {
        memcmp: {
          offset: 8,
          bytes: alice.publicKey.toBase58(),
        },
      },
    ]);

    await Promise.all(
      aliceClaimTickets.map((claimTicket) =>
        program.methods
          .claim()
          .accounts({
            owner: alice.publicKey,
            claimTicket: claimTicket.publicKey,
            authority,
            gsrmMint: GSRM_MINT,
            ownerGsrmAccount: aliceGSRMAccount.publicKey,
            clock: SYSVAR_CLOCK_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([alice])
          .rpc()
      )
    );

    const aliceGsrmBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount.publicKey
    );

    expect(aliceGsrmBalance.value.uiAmount).to.equal(1_000_000 + 200);

    aliceClaimTickets = await program.account.claimTicket.all([
      {
        memcmp: {
          offset: 8,
          bytes: alice.publicKey.toBase58(),
        },
      },
    ]);
    expect(aliceClaimTickets.length).to.equal(0);
  });

  it("can burn gsrm for locked srm", async () => {
    const lockIndex = new BN(0);
    const [aliceLockedAccount] = findProgramAddressSync(
      [
        Buffer.from("locked_account"),
        alice.publicKey.toBuffer(),
        Buffer.from(lockIndex.toBuffer("le", 8)),
      ],
      program.programId
    );

    const redeemTicket = Keypair.generate();

    await program.methods
      .burnLockedGsrm(new BN(lockIndex), new BN(100_000_000))
      .accounts({
        owner: alice.publicKey,
        authority,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount.publicKey,
        lockedAccount: aliceLockedAccount,
        redeemTicket: redeemTicket.publicKey,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice, redeemTicket])
      .rpc();

    const aliceRedeemTicket = await program.account.redeemTicket.fetch(
      redeemTicket.publicKey
    );
    expect(aliceRedeemTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceRedeemTicket.isMsrm).to.equal(false);
    expect(aliceRedeemTicket.redeemDelay.toNumber()).to.equal(2);
    expect(aliceRedeemTicket.amount.toNumber()).to.equal(100_000_000);

    const aliceGSRMBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount.publicKey
    );
    expect(aliceGSRMBalance.value.uiAmount).to.equal(1_000_100);
  });

  it("cant burn gsrm if amount exceeds locked amount", async () => {
    // Here, user has more than 200 gSRM because , but the LockedAccount has only 100 gSRM left to be redeemed.
    const lockIndex = 0;
    const [aliceLockedAccount] = findProgramAddressSync(
      [
        Buffer.from("locked_account"),
        alice.publicKey.toBuffer(),
        Buffer.from(lockIndex.toString()),
      ],
      program.programId
    );

    const redeemTicket = Keypair.generate();

    try {
      await program.methods
        .burnLockedGsrm(new BN(lockIndex), new BN(200_000_000))
        .accounts({
          owner: alice.publicKey,
          authority,
          gsrmMint: GSRM_MINT,
          ownerGsrmAccount: aliceGSRMAccount.publicKey,
          lockedAccount: aliceLockedAccount,
          redeemTicket: redeemTicket.publicKey,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice, redeemTicket])
        .rpc();
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else assert(false);
    }
  });

  it("cant burn gsrm for locked msrm with invalid amount", async () => {
    const lockIndex = 1;
    const [aliceLockedAccount] = findProgramAddressSync(
      [
        Buffer.from("locked_account"),
        alice.publicKey.toBuffer(),
        Buffer.from(lockIndex.toString()),
      ],
      program.programId
    );

    const redeemTicket = Keypair.generate();

    try {
      await program.methods
        .burnLockedGsrm(new BN(lockIndex), new BN(MSRM_MULTIPLIER - 1_000))
        .accounts({
          owner: alice.publicKey,
          authority,
          gsrmMint: GSRM_MINT,
          ownerGsrmAccount: aliceGSRMAccount.publicKey,
          lockedAccount: aliceLockedAccount,
          redeemTicket: redeemTicket.publicKey,
          clock: SYSVAR_CLOCK_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([alice, redeemTicket])
        .rpc();
    } catch (e) {
      if (e instanceof AnchorError) {
        assert(true, e.error.errorMessage);
      } else assert(false);
    }
  });

  it("can burn gsrm for locked msrm", async () => {
    const lockIndex = new BN(1);
    const [aliceLockedAccount] = findProgramAddressSync(
      [
        Buffer.from("locked_account"),
        alice.publicKey.toBuffer(),
        Buffer.from(lockIndex.toBuffer("le", 8)),
      ],
      program.programId
    );

    const redeemTicket = Keypair.generate();

    await program.methods
      .burnLockedGsrm(new BN(lockIndex), new BN(1 * MSRM_MULTIPLIER))
      .accounts({
        owner: alice.publicKey,
        authority,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount.publicKey,
        lockedAccount: aliceLockedAccount,
        redeemTicket: redeemTicket.publicKey,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice, redeemTicket])
      .rpc();

    const aliceRedeemTicket = await program.account.redeemTicket.fetch(
      redeemTicket.publicKey
    );
    expect(aliceRedeemTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceRedeemTicket.isMsrm).to.equal(true);
    expect(aliceRedeemTicket.redeemDelay.toNumber()).to.equal(2);
    expect(aliceRedeemTicket.amount.toNumber()).to.equal(1);

    const aliceGSRMBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount.publicKey
    );
    expect(aliceGSRMBalance.value.uiAmount).to.equal(100);

    // This should also close the LockedAccount
    try {
      await program.account.lockedAccount.fetch(aliceLockedAccount);
      assert(false);
    } catch (e) {
      assert(true);
    }
  });

  it("can redeem ticket for srm", async () => {
    await sleep(2);
    let redeemTickets = await program.account.redeemTicket.all([
      {
        memcmp: {
          offset: 8,
          bytes: alice.publicKey.toBase58(),
        },
      },
    ]);

    const [redeemTicket] = redeemTickets.filter(
      (ticket) => ticket.account.isMsrm === false
    );

    await program.methods
      .redeemSrm()
      .accounts({
        owner: alice.publicKey,
        authority,
        redeemTicket: redeemTicket.publicKey,
        srmMint: SRM_MINT,
        srmVault,
        ownerSrmAccount: aliceSRMAccount.publicKey,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceSrmBalance = await connection.getTokenAccountBalance(
      aliceSRMAccount.publicKey
    );

    expect(aliceSrmBalance.value.uiAmount).to.equal(50100);

    redeemTickets = await program.account.redeemTicket.all([
      {
        memcmp: {
          offset: 8,
          bytes: alice.publicKey.toBase58(),
        },
      },
    ]);

    expect(redeemTickets.length).to.equal(1);
  });

  it("can redeem ticket for msrm", async () => {
    // Since only one ticket should be remaining
    const [redeemTicket] = await program.account.redeemTicket.all([
      {
        memcmp: {
          offset: 8,
          bytes: alice.publicKey.toBase58(),
        },
      },
    ]);

    await program.methods
      .redeemMsrm()
      .accounts({
        owner: alice.publicKey,
        authority,
        redeemTicket: redeemTicket.publicKey,
        msrmMint: MSRM_MINT,
        msrmVault,
        ownerMsrmAccount: aliceMSRMAccount.publicKey,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceMsrmBalance = await connection.getTokenAccountBalance(
      aliceMSRMAccount.publicKey
    );

    expect(aliceMsrmBalance.value.uiAmount).to.equal(2);

    const redeemTickets = await program.account.redeemTicket.all([
      {
        memcmp: {
          offset: 8,
          bytes: alice.publicKey.toBase58(),
        },
      },
    ]);

    expect(redeemTickets.length).to.equal(0);
  });

  it("can deposit vest srm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [aliceVestAccount] = findProgramAddressSync(
      [
        Buffer.from("vest_account"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.vestIndex.toBuffer("le", 8)),
      ],
      program.programId
    );

    const claimTicket = Keypair.generate();

    await program.methods
      .depositVestSrm(new BN(40000 * 1000000))
      .accounts({
        payer: alice.publicKey,
        owner: alice.publicKey,
        ownerUserAccount: aliceUserAccount,
        vestAccount: aliceVestAccount,
        claimTicket: claimTicket.publicKey,
        srmMint: SRM_MINT,
        payerSrmAccount: aliceSRMAccount.publicKey,
        authority,
        srmVault,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice, claimTicket])
      .rpc();

    const vestAccount = await program.account.vestAccount.fetch(
      aliceVestAccount
    );
    expect(vestAccount.owner.toBase58()).to.equal(alice.publicKey.toBase58());
    expect(vestAccount.totalGsrmAmount.toNumber()).to.equal(40000 * 1000000);

    const aliceClaimTicket = await program.account.claimTicket.fetch(
      claimTicket.publicKey
    );
    expect(aliceClaimTicket.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceClaimTicket.gsrmAmount.toNumber()).to.equal(40000 * 1000000);
    expect(aliceClaimTicket.claimDelay.toNumber()).to.equal(2);
  });

  // ------------------------------------------------------------

  // it("can burn vest gsrm", async () => {
  //   await sleep(2);
  //   const [claimTicket] = await program.account.claimTicket.all([
  //     {
  //       memcmp: {
  //         offset: 8,
  //         bytes: alice.publicKey.toBase58(),
  //       },
  //     },
  //   ]);

  //   await program.methods
  //     .claim()
  //     .accounts({
  //       owner: alice.publicKey,
  //       claimTicket: claimTicket.publicKey,
  //       authority,
  //       gsrmMint: GSRM_MINT,
  //       ownerGsrmAccount: aliceGSRMAccount,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice])
  //     .rpc();

  //   const [aliceVestAccount] = findProgramAddressSync(
  //     [
  //       Buffer.from("vest_account"),
  //       alice.publicKey.toBuffer(),
  //       Buffer.from("0"),
  //     ],
  //     program.programId
  //   );

  //   await sleep(10);
  //   const redeemTicket = Keypair.generate();
  //   const sig = await program.methods
  //     .burnVestGsrm(new BN(0), new BN(100_000_000_000))
  //     .accounts({
  //       owner: alice.publicKey,
  //       authority,
  //       gsrmMint: GSRM_MINT,
  //       ownerGsrmAccount: aliceGSRMAccount,
  //       vestAccount: aliceVestAccount,
  //       redeemTicket: redeemTicket.publicKey,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice, redeemTicket])
  //     .rpc();
  //   console.log(sig);

  //   await sleep(10);
  //   const redeemTicket2 = Keypair.generate();
  //   const sig2 = await program.methods
  //     .burnVestGsrm(new BN(0), new BN(100_000_000_000))
  //     .accounts({
  //       owner: alice.publicKey,
  //       authority,
  //       gsrmMint: GSRM_MINT,
  //       ownerGsrmAccount: aliceGSRMAccount,
  //       vestAccount: aliceVestAccount,
  //       redeemTicket: redeemTicket2.publicKey,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice, redeemTicket2])
  //     .rpc();
  //   console.log(sig2);
  // });
});
