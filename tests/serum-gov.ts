import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
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

  const sbf = Keypair.generate();
  const alice = Keypair.generate();

  let aliceSRMAccount: PublicKey;
  let aliceMSRMAccount: PublicKey;

  const [authority] = findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  );
  const [gsrmMint] = findProgramAddressSync(
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

  it("can init!", async () => {
    const tx = await program.methods
      .init()
      .accounts({
        payer: sbf.publicKey,
        authority,
        gsrmMint,
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

    const mint = await getMint(connection, gsrmMint);
    expect(mint.decimals).to.equal(9);
    expect(mint.mintAuthority.toBase58()).to.equal(sbf.publicKey.toBase58());

    const vaultSrm = await connection.getTokenAccountBalance(srmVault);
    expect(vaultSrm.value.uiAmount).to.equal(0);

    const vaultMsrm = await connection.getTokenAccountBalance(msrmVault);
    expect(vaultMsrm.value.uiAmount).to.equal(0);
  });

  it("cant init vaults again!", async () => {
    try {
      await program.methods
        .init()
        .accounts({
          payer: sbf.publicKey,
          authority,
          gsrmMint,
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
    expect(aliceAccount.lockerIndex.toNumber()).to.equal(0);
  });

  it("can deposit srm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [locker] = findProgramAddressSync(
      [
        Buffer.from("locker"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.lockerIndex.toString()),
      ],
      program.programId
    );

    await program.methods
      .depositSrm(new BN(100_000_000), new BN(0), new BN(1_000))
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        srmMint: SRM_MINT,
        ownerSrmAccount: aliceSRMAccount,
        authority,
        srmVault,
        locker,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc();

    const aliceLocker = await program.account.locker.fetch(locker);
    const srmVaultBalance = await connection.getTokenAccountBalance(srmVault);

    expect(aliceLocker.owner.toBase58()).to.equal(alice.publicKey.toBase58());
    expect(aliceLocker.lockerIndex.toNumber()).to.equal(0);
    expect(aliceLocker.amount.toNumber()).to.equal(100_000_000);
    expect(aliceLocker.isMsrm).to.equal(false);
    expect(aliceLocker.collectDelay.toNumber()).to.equal(0);
    expect(aliceLocker.redeemDelay.toNumber()).to.equal(1_000);

    expect(srmVaultBalance.value.uiAmount).to.equal(100);
  });

  it("can deposit msrm", async () => {
    const aliceAccount = await program.account.user.fetch(aliceUserAccount);
    const [msrmLocker] = findProgramAddressSync(
      [
        Buffer.from("locker"),
        alice.publicKey.toBuffer(),
        Buffer.from(aliceAccount.lockerIndex.toString()),
      ],
      program.programId
    );

    await program.methods
      .depositMsrm(new BN(1), new BN(0), new BN(1_000))
      .accounts({
        owner: alice.publicKey,
        userAccount: aliceUserAccount,
        msrmMint: MSRM_MINT,
        ownerMsrmAccount: aliceMSRMAccount,
        authority,
        msrmVault,
        locker: msrmLocker,
        clock: SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([alice])
      .rpc({ skipPreflight: true });

    const aliceMSRMLocker = await program.account.locker.fetch(msrmLocker);
    console.log(aliceMSRMLocker);
    const msrmVaultBalance = await connection.getTokenAccountBalance(msrmVault);

    expect(aliceMSRMLocker.owner.toBase58()).to.equal(
      alice.publicKey.toBase58()
    );
    expect(aliceMSRMLocker.lockerIndex.toNumber()).to.equal(1);
    expect(aliceMSRMLocker.amount.toNumber()).to.equal(1);
    expect(aliceMSRMLocker.isMsrm).to.equal(true);
    expect(aliceMSRMLocker.collectDelay.toNumber()).to.equal(0);
    expect(aliceMSRMLocker.redeemDelay.toNumber()).to.equal(1_000);

    expect(msrmVaultBalance.value.uiAmount).to.equal(1);
  });
});
