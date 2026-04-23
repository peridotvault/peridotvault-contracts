import { expect } from "chai";
import { SystemProgram } from "@solana/web3.js";
import {
  configureStoreForGame,
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("native SOL-style payment mint flow", () => {
  it("allows listing with system program mint placeholder", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

    const listing = await configureStoreForGame(base, game, {
      basePrice: 50_000,
      paymentMint: SystemProgram.programId,
      active: true,
    });

    const paymentOption = (await base.storeProgram.account.gamePaymentOption.fetch(
      listing.gamePaymentOptionPda,
    )) as any;
    expect(paymentOption.mint.toBase58()).to.eq(SystemProgram.programId.toBase58());
    expect(paymentOption.active).to.eq(true);
  });
});
