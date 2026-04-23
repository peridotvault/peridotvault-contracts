import { expect } from "chai";
import {
  DEFAULT_GAME_PRICE,
  buyGameForBuyer,
  configureStoreForGame,
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("gamer purchase flow", () => {
  it("buys game, settles payment, and mints license", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);
    await configureStoreForGame(base, game, {
      basePrice: DEFAULT_GAME_PRICE,
      active: true,
    });

    const result = await buyGameForBuyer(base, game, DEFAULT_GAME_PRICE, {
      buyer: base.gamer,
    });

    const receipt = (await base.storeProgram.account.purchaseReceipt.fetch(
      result.purchaseReceiptPda,
    )) as any;
    expect(receipt.buyer.toBase58()).to.eq(base.gamer.publicKey.toBase58());
    expect(receipt.game.toBase58()).to.eq(game.gamePda.toBase58());
    expect(receipt.finalPrice.toString()).to.eq(DEFAULT_GAME_PRICE.toString());

    const licensePda = (receipt: any) =>
      base.pglProgram.account.license
        .all([
          {
            memcmp: {
              offset: 8,
              bytes: receipt.buyer.toBase58(),
            },
          },
        ])
        .then((licenses: any[]) =>
          licenses.find((l) => l.account.game.toBase58() === receipt.game.toBase58()),
        );

    const owned = await licensePda(receipt);
    expect(owned).to.not.eq(undefined);
    const license = owned!.account;
    expect(license.holder.toBase58()).to.eq(base.gamer.publicKey.toBase58());
    expect(license.game.toBase58()).to.eq(game.gamePda.toBase58());
  });

  it("keeps license ownership single for repeated buy attempts", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `double-buy-${Date.now()}`,
    });
    await configureStoreForGame(base, game, {
      basePrice: DEFAULT_GAME_PRICE,
      active: true,
    });

    const first = await buyGameForBuyer(base, game, DEFAULT_GAME_PRICE, {
      buyer: base.gamer,
    });

    let failed = false;
    try {
      await buyGameForBuyer(base, game, DEFAULT_GAME_PRICE, {
        buyer: base.gamer,
      });
    } catch (error: any) {
      failed = true;
      const err = String(error);
      expect(
        err.includes("Already owned") ||
          err.includes("AlreadyOwned") ||
          err.includes("custom program error: 0x1774") ||
          err.includes("already in use") ||
          err.includes("Constraint"),
      ).to.eq(true);
    }

    const receipts = await base.storeProgram.account.purchaseReceipt.all([
      {
        memcmp: {
          offset: 8,
          bytes: base.gamer.publicKey.toBase58(),
        },
      },
      {
        memcmp: {
          offset: 40,
          bytes: game.gamePda.toBase58(),
        },
      },
    ]);
    expect(receipts.length).to.eq(1);
    expect(receipts[0].publicKey.toBase58()).to.eq(first.purchaseReceiptPda.toBase58());

    const licenses = await base.pglProgram.account.license.all([
      {
        memcmp: {
          offset: 8,
          bytes: base.gamer.publicKey.toBase58(),
        },
      },
      {
        memcmp: {
          offset: 40,
          bytes: game.gamePda.toBase58(),
        },
      },
    ]);
    expect(licenses.length).to.eq(1);
    if (!failed) {
      const receipt = receipts[0].account as any;
      expect(receipt.finalPrice.toString()).to.eq(DEFAULT_GAME_PRICE.toString());
    }
  });
});
