import { expect } from "chai";
import {
  STATUS_APPROVED,
  TEST_GAME_ID,
  DEFAULT_GAME_PRICE,
  ensurePriceConfigured,
  getCatalogWithPrices,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("publisher registry and store views", () => {
  it("verify direct price account and catalog listing", async () => {
    const base = await setupPeridotFixture();
    await ensurePriceConfigured(base);

    const catalog = await getCatalogWithPrices(base);
    const catalogGame = catalog.find((entry) => entry.gameId === TEST_GAME_ID);

    expect(catalogGame).to.not.equal(undefined);
    expect(catalogGame!.status).to.equal(STATUS_APPROVED);
    expect(catalogGame!.price).to.equal(DEFAULT_GAME_PRICE);
    expect(catalogGame!.finalPrice).to.equal(DEFAULT_GAME_PRICE);
  });
});
