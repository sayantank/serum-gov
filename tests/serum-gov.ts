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

  let aliceSRMAccount: PublicKey;
  let aliceMSRMAccount: PublicKey;
  let aliceGSRMAccount: PublicKey;

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
    expect(aliceAccount.lockIndex.toNumber()).to.equal(0);
  });

  it("can deposit srm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [aliceLockedAccount] = findProgramAddressSync(
      [
        Buffer.from("locked_account"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.lockIndex.toString()),
      ],
      program.programId
    );

    const claimTicket = Keypair.generate();

    await program.methods
      .depositLockedSrm(new BN(200_000_000))
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        srmMint: SRM_MINT,
        ownerSrmAccount: aliceSRMAccount,
        authority,
        srmVault,
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
        Buffer.from(aliceAccount.lockIndex.toString()),
      ],
      program.programId
    );

    let claimTicket = Keypair.generate();

    await program.methods
      .depositLockedMsrm(new BN(1))
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        msrmMint: MSRM_MINT,
        ownerMsrmAccount: aliceMSRMAccount,
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
    await sleep(2);

    aliceGSRMAccount = await createAssociatedTokenAccount(
      connection,
      alice,
      GSRM_MINT,
      alice.publicKey
    );

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
            ownerGsrmAccount: aliceGSRMAccount,
            clock: SYSVAR_CLOCK_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([alice])
          .rpc()
      )
    );

    const aliceGsrmBalance = await connection.getTokenAccountBalance(
      aliceGSRMAccount
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

    await program.methods
      .burnLockedGsrm(new BN(lockIndex), new BN(100_000_000))
      .accounts({
        owner: alice.publicKey,
        authority,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount,
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
      aliceGSRMAccount
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
          ownerGsrmAccount: aliceGSRMAccount,
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
          ownerGsrmAccount: aliceGSRMAccount,
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

    await program.methods
      .burnLockedGsrm(new BN(lockIndex), new BN(1 * MSRM_MULTIPLIER))
      .accounts({
        owner: alice.publicKey,
        authority,
        gsrmMint: GSRM_MINT,
        ownerGsrmAccount: aliceGSRMAccount,
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
      aliceGSRMAccount
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

  // it("cant redeem msrm with srm ticket", async () => {
  //   const [redeemTicket] = findProgramAddressSync(
  //     [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("0")],
  //     program.programId
  //   );

  //   try {
  //     await program.methods
  //       .redeemMsrm(new BN(1))
  //       .accounts({
  //         owner: alice.publicKey,
  //         authority,
  //         redeemTicket,
  //         msrmMint: MSRM_MINT,
  //         msrmVault,
  //         ownerMsrmAccount: aliceMSRMAccount,
  //         clock: SYSVAR_CLOCK_PUBKEY,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([alice])
  //       .rpc();
  //   } catch (e) {
  //     if (e instanceof AnchorError) {
  //       assert(true, e.error.errorMessage);
  //     } else {
  //       console.error(e);
  //       assert(false);
  //     }
  //   }
  // });

  // it("can redeem srm", async () => {
  //   await sleep(2);

  //   const [redeemTicket] = findProgramAddressSync(
  //     [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("0")],
  //     program.programId
  //   );

  //   await program.methods
  //     .redeemSrm(new BN(0))
  //     .accounts({
  //       owner: alice.publicKey,
  //       authority,
  //       redeemTicket,
  //       srmMint: SRM_MINT,
  //       srmVault,
  //       ownerSrmAccount: aliceSRMAccount,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice])
  //     .rpc();

  //   const aliceSrmBalance = await connection.getTokenAccountBalance(
  //     aliceSRMAccount
  //   );
  //   expect(aliceSrmBalance.value.uiAmount).to.equal(100);

  //   try {
  //     await program.account.redeemTicket.fetch(redeemTicket);
  //     assert(false);
  //   } catch (e) {
  //     if (e instanceof Error) {
  //       assert(true);
  //     }
  //   }
  // });

  // it("cant redeem srm with msrm ticket", async () => {
  //   const [redeemTicket] = findProgramAddressSync(
  //     [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("1")],
  //     program.programId
  //   );

  //   try {
  //     await program.methods
  //       .redeemSrm(new BN(1))
  //       .accounts({
  //         owner: alice.publicKey,
  //         authority,
  //         redeemTicket,
  //         srmMint: SRM_MINT,
  //         srmVault,
  //         ownerSrmAccount: aliceSRMAccount,
  //         clock: SYSVAR_CLOCK_PUBKEY,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([alice])
  //       .rpc();
  //   } catch (e) {
  //     if (e instanceof AnchorError) {
  //       assert(true, e.error.errorMessage);
  //     } else {
  //       console.error(e);
  //       assert(false);
  //     }
  //   }
  // });

  // it("can redeem msrm", async () => {
  //   const [redeemTicket] = findProgramAddressSync(
  //     [Buffer.from("redeem"), alice.publicKey.toBuffer(), Buffer.from("1")],
  //     program.programId
  //   );

  //   await program.methods
  //     .redeemMsrm(new BN(1))
  //     .accounts({
  //       owner: alice.publicKey,
  //       authority,
  //       redeemTicket,
  //       msrmMint: MSRM_MINT,
  //       msrmVault,
  //       ownerMsrmAccount: aliceMSRMAccount,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice])
  //     .rpc();

  //   const aliceMsrmBalance = await connection.getTokenAccountBalance(
  //     aliceMSRMAccount
  //   );
  //   expect(aliceMsrmBalance.value.uiAmount).to.equal(2);

  //   try {
  //     await program.account.redeemTicket.fetch(redeemTicket);
  //     assert(false);
  //   } catch (e) {
  //     if (e instanceof Error) {
  //       assert(true);
  //     }
  //   }
  // });

  // it("cant claim before claim_delay", async () => {
  //   const aliceAccount = await program.account.user.fetch(aliceUserAccount);
  //   const [claimTicket] = findProgramAddressSync(
  //     [
  //       Buffer.from("claim"),
  //       alice.publicKey.toBuffer(),
  //       Buffer.from(aliceAccount.claimIndex.toString()),
  //     ],
  //     program.programId
  //   );

  //   await program.methods
  //     .depositSrm(new BN(100_000_000))
  //     .accounts({
  //       owner: alice.publicKey,
  //       userAccount: aliceUserAccount,
  //       srmMint: SRM_MINT,
  //       ownerSrmAccount: aliceSRMAccount,
  //       authority,
  //       srmVault,
  //       claimTicket,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice])
  //     .rpc();

  //   try {
  //     await program.methods
  //       .claim(aliceAccount.claimIndex)
  //       .accounts({
  //         owner: alice.publicKey,
  //         ticket: claimTicket,
  //         authority,
  //         gsrmMint: GSRM_MINT,
  //         ownerGsrmAccount: aliceGSRMAccount,
  //         clock: SYSVAR_CLOCK_PUBKEY,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([alice])
  //       .rpc();
  //   } catch (e) {
  //     if (e instanceof AnchorError) {
  //       assert(true, e.error.errorMessage);
  //     } else {
  //       console.error(e);
  //       assert(false);
  //     }
  //   }
  // });

  // it("cant redeem before redeem_delay", async () => {
  //   const aliceAccount = await program.account.user.fetch(aliceUserAccount);
  //   const [redeemTicket] = findProgramAddressSync(
  //     [
  //       Buffer.from("redeem"),
  //       alice.publicKey.toBuffer(),
  //       Buffer.from(aliceAccount.redeemIndex.toString()),
  //     ],
  //     program.programId
  //   );

  //   await program.methods
  //     .burnGsrm(new BN(100_000_000), false)
  //     .accounts({
  //       owner: alice.publicKey,
  //       userAccount: aliceUserAccount,
  //       authority,
  //       gsrmMint: GSRM_MINT,
  //       ownerGsrmAccount: aliceGSRMAccount,
  //       redeemTicket,
  //       clock: SYSVAR_CLOCK_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([alice])
  //     .rpc();

  //   try {
  //     await program.methods
  //       .redeemSrm(aliceAccount.redeemIndex)
  //       .accounts({
  //         owner: alice.publicKey,
  //         authority,
  //         redeemTicket,
  //         srmMint: SRM_MINT,
  //         srmVault,
  //         ownerSrmAccount: aliceSRMAccount,
  //         clock: SYSVAR_CLOCK_PUBKEY,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([alice])
  //       .rpc();
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
