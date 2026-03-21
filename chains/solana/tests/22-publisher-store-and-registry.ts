import { expect } from "chai";

import {
  DEFAULT_GAME_DISCOUNT_BPS,
  DEFAULT_GAME_PRICE,
  STATUS_APPROVED,
  TEST_GAME_ID,
  approveGame,
  ensurePriceConfigured,
  getCatalogWithPrices,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("publisher registry and store views", () => {
  it("approves the game, sets price and discount, and lists games with pricing", async () => {
    const base = await setupPeridotFixture();
    await ensurePriceConfigured(base);
    await approveGame(base, TEST_GAME_ID);

    const registryState = (await base.registryProgram.account.registryState.fetch(
      base.registryStatePda,
    )) as any;
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const registryGame = registryState.games.find((entry: any) => entry.gameId === TEST_GAME_ID);
    const priceConfig = storeState.prices.find((entry: any) => entry.gameId === TEST_GAME_ID);
    const catalog = await getCatalogWithPrices(base);
    const catalogGame = catalog.find((entry) => entry.gameId === TEST_GAME_ID);

    expect(registryGame.status).to.equal(STATUS_APPROVED);
    expect(Number(priceConfig.price.toString())).to.equal(DEFAULT_GAME_PRICE);
    expect(priceConfig.discountBps).to.equal(DEFAULT_GAME_DISCOUNT_BPS);
    expect(catalogGame).to.not.equal(undefined);
    expect(catalogGame!.status).to.equal(STATUS_APPROVED);
    expect(catalogGame!.price).to.equal(DEFAULT_GAME_PRICE);
    expect(catalogGame!.discountBps).to.equal(DEFAULT_GAME_DISCOUNT_BPS);
    expect(catalogGame!.finalPrice).to.equal(17_000_000);
  });
});
