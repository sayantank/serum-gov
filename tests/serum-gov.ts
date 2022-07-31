import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  Account,
  createAssociatedTokenAccount,
  createMint,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { SerumGov } from "../target/types/serum_gov";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { assert } from "chai";

describe("serum-gov", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const { connection } = provider;
  const program = anchor.workspace.SerumGov as Program<SerumGov>;

  let SRM_MINT: PublicKey;
  let MSRM_MINT: PublicKey;

  const sbf = Keypair.generate();
  const alice = Keypair.generate();

  let aliceSRMAccount: PublicKey;
  let aliceMSRMAccount: PublicKey;

  let vaultAuthority: PublicKey;
  let srmVault: PublicKey;
  let msrmVault: PublicKey;

  before(async () => {
    const sbfDrop = await connection.requestAirdrop(
      sbf.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sbfDrop);

    const aliceDrop = await connection.requestAirdrop(
      alice.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(aliceDrop);

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

    aliceSRMAccount = await createAssociatedTokenAccount(
      connection,
      alice,
      SRM_MINT,
      alice.publicKey
    );
    aliceMSRMAccount = await createAssociatedTokenAccount(
      connection,
      alice,
      MSRM_MINT,
      alice.publicKey
    );

    await mintTo(
      connection,
      sbf,
      SRM_MINT,
      aliceSRMAccount,
      sbf,
      BigInt(200 * 1000000)
    );

    await mintTo(connection, sbf, MSRM_MINT, aliceMSRMAccount, sbf, 2);

    [vaultAuthority] = findProgramAddressSync(
      [Buffer.from("authority")],
      program.programId
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

  it("can init vaults!", async () => {
    const tx = await program.methods
      .initVaults()
      .accounts({
        payer: sbf.publicKey,
        vaultAuthority,
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
    console.log("Your transaction signature", tx);
  });

  it("cant init vaults again!", async () => {
    try {
      await program.methods
        .initVaults()
        .accounts({
          payer: sbf.publicKey,
          vaultAuthority,
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
});
