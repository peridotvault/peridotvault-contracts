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
    
    // Create game with SOL as currency
    const game = deriveGameFixture(base, gameId);
    const mintKp = Keypair.generate();
    await base.pgc1Program.methods
      .createGame(
        gameId,
        base.publisher.publicKey,
        game.metadataUri,
        new anchor.BN(10_000_000), // 0.01 SOL
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
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID, // Token-2022 handled via Interface or Address
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([base.publisher, mintKp])
      .rpc();

    // Purchase in SOL
    const balancePda = anchor.web3.PublicKey.findProgramAddressSync(
        [BALANCE_SEED, base.publisher.publicKey.toBuffer(), SystemProgram.programId.toBuffer()],
        base.storeProgram.programId
      )[0];

    const treasuryInitial = await base.provider.connection.getBalance(base.treasury.publicKey);

    await base.storeProgram.methods
      .buyGame()
      .accounts({
        buyer: base.gamer.publicKey,
        storeState: base.storeStatePda,
        treasury: base.treasury.publicKey,
        pgcGameState: game.gameStatePda,
        priceAccount: game.pricePda,
        publisherBalanceAccount: balancePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([base.gamer])
      .rpc();

    const treasuryFinal = await base.provider.connection.getBalance(base.treasury.publicKey);
    const storeState = await base.storeProgram.account.storeState.fetch(base.storeStatePda);
    const expectedFee = Math.floor((10_000_000 * storeState.platformFeeBps) / 10000);
    
    expect(treasuryFinal - treasuryInitial).to.equal(expectedFee);

    // Withdraw SOL
    const pubInitial = await base.provider.connection.getBalance(base.publisher.publicKey);
    await base.storeProgram.methods
      .withdraw(anchor.web3.PublicKey.default)
      .accounts({
        publisher: base.publisher.publicKey,
        storeState: base.storeStatePda,
        publisherBalanceAccount: balancePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([base.publisher])
      .rpc();

    const pubFinal = await base.provider.connection.getBalance(base.publisher.publicKey);
    expect(pubFinal).to.be.greaterThan(pubInitial); // Roughly +0.009 SOL minus fees
  });
});
