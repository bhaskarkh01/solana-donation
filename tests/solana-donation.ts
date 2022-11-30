import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaDonation } from "../target/types/solana_donation";

describe("solana-donation", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolanaDonation as Program<SolanaDonation>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
