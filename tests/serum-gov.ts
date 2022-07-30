import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SerumGov } from "../target/types/serum_gov";

describe("serum-gov", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SerumGov as Program<SerumGov>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
