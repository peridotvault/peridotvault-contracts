import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SystemProgram } from "@solana/web3.js";

import {
  DEFAULT_GAME_PRICE,
  STATUS_APPROVED,
  TEST_GAME_ID,
  deriveGameFixture,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("fee exempt auto approval", () => {
  it("registers fee-exempt publisher games as approved immediately", async () => {
    const base = await setupPeridotFixture();
    const game = deriveGameFixture(base);

    await base.registryProgram.methods
      .setFeeExemption(base.publisher.publicKey, true)
      .accounts({
        governance: base.governance.publicKey,
        registryState: base.registryStatePda,
      } as any)
      .rpc();

    await base.factoryProgram.methods
      .createGame(
        game.gameId,
        game.metadataUri,
        new anchor.BN(DEFAULT_GAME_PRICE),
        base.paymentMint,
        base.paymentMint,
      )
      .accounts({
        publisher: base.publisher.publicKey,
        factoryState: base.factoryStatePda,
        mint: game.mintPda,
        pgcProgram: base.pgcProgram.programId,
        pgcGameState: game.gameStatePda,
        pgcGameAuthority: game.gameAuthorityPda,
        publisherMinterAuth: game.publisherMinterAuthPda,
        gameStoreMinterAuth: game.storeMinterAuthPda,
        registryProgram: base.registryProgram.programId,
        registryState: base.registryStatePda,
        gameStoreProgram: base.storeProgram.programId,
        treasury: base.treasury.publicKey,
        gameStore: base.storeStatePda,
        publisherFeeTokenAccount: base.publisherPaymentTokenAccount,
        treasuryFeeTokenAccount: base.treasuryPaymentTokenAccount,
        feePaymentMint: base.paymentMint,
        paymentTokenProgram: TOKEN_PROGRAM_ID,
        priceCurrencyMint: base.paymentMint,
        licenseTokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([base.publisher])
      .rpc();

    const registryState = (await base.registryProgram.account.registryState.fetch(
      base.registryStatePda,
    )) as any;
    const registryGame = registryState.games.find((entry: any) => entry.gameId === TEST_GAME_ID);

    expect(registryGame.status).to.equal(STATUS_APPROVED);
  });
});
