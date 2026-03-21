import { expect } from "chai";

import {
  TEST_GAME_ID,
  buyGameForGamer,
  listOwnedGames,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("gamer library", () => {
  it("lists the owned games for the gamer", async () => {
    const base = await setupPeridotFixture();
    await buyGameForGamer(base);

    const ownedGames = await listOwnedGames(base, base.gamer.publicKey);
    const ownedGame = ownedGames.find((entry) => entry.gameId === TEST_GAME_ID);

    expect(ownedGames.length).to.be.greaterThan(0);
    expect(ownedGame).to.not.equal(undefined);
    expect(ownedGame!.contractAddress.toBase58()).to.be.a("string");
    expect(ownedGame!.finalPrice).to.equal(17_000_000);
  });
});
