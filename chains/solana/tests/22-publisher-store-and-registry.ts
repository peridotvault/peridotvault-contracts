import { expect } from "chai";
import {
  DEFAULT_GAME_PRICE,
  configureStoreForGame,
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("publisher store configuration", () => {
  it("configures listing for canonical pgl/registry game", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

    const listing = await configureStoreForGame(base, game, {
      basePrice: DEFAULT_GAME_PRICE,
      active: true,
    });

    const gameStoreConfig = (await base.storeProgram.account.gameStoreConfig.fetch(
      listing.gameStoreConfigPda,
    )) as any;
    expect(gameStoreConfig.game.toBase58()).to.eq(game.gamePda.toBase58());
    expect(gameStoreConfig.active).to.eq(true);
  });
});
