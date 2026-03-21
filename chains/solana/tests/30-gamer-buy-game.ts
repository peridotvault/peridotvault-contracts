import { expect } from "chai";

import {
  buyGameForGamer,
  ensurePriceConfigured,
  licenseTokenBalance,
  paymentTokenBalance,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("gamer purchase flow", () => {
  it("buys a game and mints the license badge", async () => {
    const base = await setupPeridotFixture();
    await ensurePriceConfigured(base);

    const treasuryBalanceBefore = await paymentTokenBalance(
      base,
      base.treasuryPaymentTokenAccount,
    );
    const purchase = await buyGameForGamer(base);
    const treasuryBalanceAfter = await paymentTokenBalance(base, base.treasuryPaymentTokenAccount);

    const license = (await base.pgcProgram.account.licenseAccount.fetch(
      purchase.licensePda,
    )) as any;
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const badgeBalance = await licenseTokenBalance(base, purchase.userGameTokenAccount);
    const priceConfig = storeState.prices.find(
      (entry: any) => entry.gameId === purchase.game.gameId,
    );
    const basePrice = Number(priceConfig.price.toString());
    const finalPrice =
      basePrice - Math.floor((basePrice * priceConfig.discountBps) / 10_000);
    const expectedPlatformFee = Math.floor(
      (finalPrice * storeState.platformFeeBps) / 10_000,
    );
    const expectedPublisherRevenue = finalPrice - expectedPlatformFee;

    expect(license.user.toBase58()).to.equal(base.gamer.publicKey.toBase58());
    expect(license.game.toBase58()).to.equal(purchase.game.gameStatePda.toBase58());
    expect(Number(license.expiresAt.toString())).to.equal(0);
    expect(license.badgeMinted).to.equal(true);
    expect(badgeBalance).to.equal(1);
    expect(treasuryBalanceAfter - treasuryBalanceBefore).to.equal(expectedPlatformFee);

    const publisherBalance = storeState.publisherBalances.find(
      (entry: any) =>
        entry.publisher.toBase58() === base.publisher.publicKey.toBase58() &&
        entry.token.toBase58() === base.paymentMint.toBase58(),
    );
    expect(Number(publisherBalance.amount.toString())).to.equal(expectedPublisherRevenue);
  });
});
