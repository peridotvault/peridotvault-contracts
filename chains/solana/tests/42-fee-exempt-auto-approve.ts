import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
import {
  STATUS_APPROVED,
  deriveGameFixture,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("fee exempt auto approval", () => {
  it("registers fee-exempt publisher games as approved immediately", async () => {
    const base = await setupPeridotFixture();
    const gameId = "exempt-game-" + Math.floor(Math.random() * 1000000);
    const game = deriveGameFixture(base, gameId);
    
    // Add publisher to exemption list
    await base.registryProgram.methods
      .setFeeExemption(base.publisher.publicKey, true)
      .accounts({
        admin: base.governance.publicKey,
        registryState: base.registryStatePda,
        account: base.publisher.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([base.governance])
      .rpc();

    const mintKp = Keypair.generate();
    await base.pgcProgram.methods
      .createGame(
        gameId,
        base.publisher.publicKey,
        game.metadataUri,
        new anchor.BN(0),
        SystemProgram.programId,
      )
      .accounts({
        payer: base.publisher.publicKey,
        mint: mintKp.publicKey,
        gameState: game.gameStatePda,
        gameAuthority: game.gameAuthorityPda,
        publisherAccount: base.publisher.publicKey,
        publisherMinterAuth: game.publisherMinterAuthPda,
        globalState: base.pgcGlobalStatePda,
        registryProgram: base.registryProgram.programId,
        registryState: base.registryStatePda,
        gameRegistration: game.gameRegistrationPda,
        gameStoreProgram: base.storeProgram.programId,
        storeState: base.storeStatePda,
        priceAccount: game.pricePda,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([base.publisher, mintKp])
      .rpc();

    const reg = await base.registryProgram.account.gameRegistration.fetch(game.gameRegistrationPda);
    expect(reg.status).to.equal(STATUS_APPROVED);

    // Cleanup
    await base.registryProgram.methods
      .setFeeExemption(base.publisher.publicKey, false)
      .accounts({
        admin: base.governance.publicKey,
        registryState: base.registryStatePda,
      } as any)
      .signers([base.governance])
      .rpc();
  });
});
