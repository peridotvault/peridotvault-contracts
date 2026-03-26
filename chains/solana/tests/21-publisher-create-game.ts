import { expect } from "chai";

import {
  DEFAULT_GAME_PRICE,
  STATUS_PENDING,
  TEST_GAME_ID,
  TEST_METADATA_URI,
  ensureGameCreated,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("publisher factory flow", () => {
  it("creates a new game through factory and registers it as pending", async () => {
    const base = await setupPeridotFixture();
    const game = await ensureGameCreated(base);

    const pgcGameState = (await base.pgcProgram.account.gameState.fetch(game.gameStatePda)) as any;
    const registryGame = await base.registryProgram.account.gameRegistration.fetch(
      game.gameRegistrationPda,
    );
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const priceConfig = storeState.prices.find((entry: any) => entry.gameId === TEST_GAME_ID);

    expect(pgcGameState.gameId).to.equal(TEST_GAME_ID);
    expect(pgcGameState.publisher.toBase58()).to.equal(base.publisher.publicKey.toBase58());
    expect(pgcGameState.metadataUri).to.equal(TEST_METADATA_URI);
    expect(registryGame.contractAddress.toBase58()).to.equal(game.gameStatePda.toBase58());
    expect(registryGame.status).to.equal(STATUS_PENDING);
    expect(Number(priceConfig.price.toString())).to.equal(DEFAULT_GAME_PRICE);
    expect(priceConfig.discountBps).to.equal(0);
  });
});
