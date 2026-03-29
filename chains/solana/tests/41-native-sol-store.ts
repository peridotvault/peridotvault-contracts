import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { Keypair, SystemProgram } from "@solana/web3.js";
import {
  BALANCE_SEED,
  buyGameForGamer,
  ensurePriceConfigured,
  setupPeridotFixture,
  deriveGameFixture,
} from "./helpers/peridot";

describe("native SOL store flow", () => {
  it("supports pricing, buying, and publisher withdrawal in SOL", async () => {
    const base = await setupPeridotFixture();
    const gameId = "sol-game-" + Math.floor(Math.random() * 1000000);
    
    // Use helper to create and approve game
    const game = await ensurePriceConfigured(base); // This creates 'peridot-localnet-alpha' by default, maybe use gameId?
    // Actually peridot.ts ensureGameCreated uses TEST_GAME_ID.
    
    const buyResult = await buyGameForGamer(base);
    const balancePda = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("balance"), base.publisher.publicKey.toBuffer(), SystemProgram.programId.toBuffer()],
        base.storeProgram.programId
      )[0];

    // Withdraw SOL
    const pubInitial = await base.provider.connection.getBalance(base.publisher.publicKey);
    await base.storeProgram.methods
      .withdraw()
      .accounts({
        authority: base.publisher.publicKey,
        config: base.storeStatePda,
        publisherBalance: balancePda,
        tokenProgram: anchor.web3.SystemProgram.programId, // Placeholder for SOL
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([base.publisher])
      .rpc();

    const pubFinal = await base.provider.connection.getBalance(base.publisher.publicKey);
    expect(pubFinal).to.be.greaterThan(pubInitial); 
  });
});
