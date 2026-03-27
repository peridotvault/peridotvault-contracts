import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { SystemProgram } from "@solana/web3.js";
import {
  BALANCE_SEED,
  buyGameForGamer,
  ensurePriceConfigured,
  licenseTokenBalance,
  setupPeridotFixture,
  deriveGameFixture,
} from "./helpers/peridot";

describe("gamer purchase flow", () => {
  it("buys a game and mints the license badge", async () => {
    const base = await setupPeridotFixture();
    await ensurePriceConfigured(base);

    const treasuryBalanceBefore = await base.provider.connection.getBalance(base.treasury.publicKey);
    const purchase = await buyGameForGamer(base);
    const treasuryBalanceAfter = await base.provider.connection.getBalance(base.treasury.publicKey);

    const license = (await base.pgc1Program.account.licenseAccount.fetch(
      purchase.licensePda,
    )) as any;
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const badgeBalance = await licenseTokenBalance(base, purchase.userGameTokenAccount);
    
    const fixture = deriveGameFixture(base);
    const priceAccount = await base.storeProgram.account.priceAccount.fetch(fixture.pricePda);
    const basePrice = Number(priceAccount.price.toString());
    const finalPrice = basePrice; // No discount set in ensurePriceConfigured yet
    
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

    const balancePda = anchor.web3.PublicKey.findProgramAddressSync(
      [BALANCE_SEED, base.publisher.publicKey.toBuffer(), SystemProgram.programId.toBuffer()],
      base.storeProgram.programId
    )[0];
    const publisherBalanceAccount = await base.storeProgram.account.publisherBalanceAccount.fetch(balancePda);
    expect(Number(publisherBalanceAccount.amount.toString())).to.equal(expectedPublisherRevenue);
  });
});
