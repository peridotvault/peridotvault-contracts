import { expect } from "chai";
import {
  DEFAULT_GAME_PRICE,
  configureStoreForGame,
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("publisher store configuration", () => {
  it("currently rejects external game/registry mirrors due owner mismatch", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

    let failed = false;
    try {
      await configureStoreForGame(base, game, {
        basePrice: DEFAULT_GAME_PRICE,
        active: true,
      });
    } catch (error: any) {
      failed = true;
      expect(String(error)).to.include("AccountOwnedByWrongProgram");
    }

    expect(failed).to.eq(true);
  });
});
