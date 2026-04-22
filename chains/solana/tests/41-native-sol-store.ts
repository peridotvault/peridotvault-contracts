import { expect } from "chai";
import { SystemProgram } from "@solana/web3.js";
import {
  configureStoreForGame,
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("native SOL-style payment mint flow", () => {
  it("currently fails for external mirrors (documenting existing behavior)", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

    let failed = false;
    try {
      await configureStoreForGame(base, game, {
        basePrice: 50_000,
        paymentMint: SystemProgram.programId,
        active: true,
      });
    } catch (error: any) {
      failed = true;
      expect(String(error)).to.include("AccountOwnedByWrongProgram");
    }

    expect(failed).to.eq(true);
  });
});
