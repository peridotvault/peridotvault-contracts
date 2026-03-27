import { expect } from "chai";
import {
  DEFAULT_GAME_PRICE,
  STATUS_APPROVED,
  TEST_GAME_ID,
  TEST_METADATA_URI,
  ensureGameCreated,
  setupPeridotFixture,
  deriveGameFixture,
} from "./helpers/peridot";

describe("publisher factory flow", () => {
  it("creates a new game through pgc1 orchestration and registers it as approved", async () => {
    const base = await setupPeridotFixture();
    const game = await ensureGameCreated(base);
    const fixture = deriveGameFixture(base, TEST_GAME_ID);

    const pgcGameState = (await base.pgcProgram.account.gameState.fetch(game.gameStatePda)) as any;
    const registryGame = await base.registryProgram.account.gameRegistration.fetch(
      game.gameRegistrationPda,
    );
    const priceAccount = await base.storeProgram.account.priceAccount.fetch(fixture.pricePda);

    expect(pgcGameState.gameId).to.equal(TEST_GAME_ID);
    expect(pgcGameState.publisher.toBase58()).to.equal(base.publisher.publicKey.toBase58());
    expect(pgcGameState.metadataUri).to.equal(TEST_METADATA_URI);
    expect(registryGame.status).to.equal(STATUS_APPROVED); // Auto-approved from PGC1
    expect(Number(priceAccount.price.toString())).to.equal(DEFAULT_GAME_PRICE);
  });
});
