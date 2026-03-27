import { expect } from "chai";
import { buyGameForGamer, setupPeridotFixture } from "./helpers/peridot";

describe("gamer library", () => {
  it("lists the owned games for the gamer", async () => {
    const base = await setupPeridotFixture();
    const purchase = await buyGameForGamer(base);

    // Filter by user in license account listings
    const licenses = await base.pgcProgram.account.licenseAccount.all([
      {
        memcmp: {
          offset: 8 + 32, // discriminator + owner
          bytes: base.gamer.publicKey.toBase58(),
        },
      },
    ]);

    expect(licenses.length).to.be.greaterThan(0);
    const myLicense = licenses.find(l => l.account.game.toBase58() === purchase.game.gameStatePda.toBase58());
    expect(myLicense).to.not.be.undefined;
  });
});
