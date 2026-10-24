import * as anchor from "@coral-xyz/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  buyGameForBuyer,
  configureStoreForGame,
  createRegisteredGame,
  derivePda,
  setupPeridotFixture,
} from "./helpers/peridot";

/**
 * ANT-478 (M0 impl) negative tests for H2/H3/M1 + M5 version gates.
 * Each test asserts the SPECIFIC on-chain error (AC-RV-2), not a generic revert.
 *
 * Requires localnet validator (`solana-test-validator`) + deployed programs.
 */

/**
 * ANT-483 (H3 consistency guard): deactivate the game's payment option so
 * `set_game_free_flag(true)` has no ACTIVE options left to conflict with.
 * Mirrors the accounts used by `configureStoreForGame`.
 */
async function deactivatePaymentOptionForGame(
  base: Awaited<ReturnType<typeof setupPeridotFixture>>,
  game: Awaited<ReturnType<typeof createRegisteredGame>>,
  store: Awaited<ReturnType<typeof configureStoreForGame>>,
) {
  const acceptedPaymentTokenPda = derivePda(
    [Buffer.from("accepted_payment_token"), store.paymentMint.toBuffer()],
    base.storeProgram.programId,
  );
  await base.storeProgram.methods
    .setGamePaymentOption(new anchor.BN(store.basePrice), false)
    .accounts({
      publisher: game.publisher.publicKey,
      authorized_source_program: base.authorizedSourceProgramPda,
      source_program: base.pglProgram.programId,
      authorized_registry_program: base.authorizedRegistryProgramPda,
      registry_program: base.registryProgram.programId,
      game: game.gamePda,
      registry_game: game.registryGamePda,
      game_store_config: store.gameStoreConfigPda,
      mint: store.paymentMint,
      accepted_payment_token: acceptedPaymentTokenPda,
      game_payment_option: store.gamePaymentOptionPda,
      system_program: SystemProgram.programId,
    })
    .signers([game.publisher])
    .rpc();
}
describe("M0 hardening negatives (H2/H3/M1/M5)", () => {
  it("AC-H2-1: direct mint_license without actor signature fails MissingActorSignature", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h2-direct-${Date.now()}`,
      metadataUri: "https://example.com/h2.json",
    });

    const attacker = Keypair.generate();
    const licensePda = derivePda(
      [Buffer.from("license"), attacker.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );

    let err: any = null;
    try {
      // actor = authorized store actor BUT the actor keypair does NOT sign.
      // Only the holder (attacker) signs as payer.
      await base.pglProgram.methods
        .mintLicense(null)
        .accounts({
          actor: base.authority.publicKey,
          holder: attacker.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: game.gamePda,
          license: licensePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected MissingActorSignature").to.not.be.null;
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /MissingActorSignature/,
    );
  });

  it("AC-H2-2: legitimate buy_game CPI path still mints (paid, happy path)", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h2-cpi-${Date.now()}`,
      metadataUri: "https://example.com/h2cpi.json",
    });
    await configureStoreForGame(base, game);

    const { buyer } = await buyGameForBuyer(base, game, base.paymentMint);
    const licensePda = derivePda(
      [Buffer.from("license"), buyer.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );
    const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
    expect(license.holder.toBase58()).to.eq(buyer.publicKey.toBase58());
    expect(Number(license.version)).to.eq(1);
  });

  it("AC-H3-1: paid game + mint_token=None fails FreePathNotAllowed", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h3-paid-${Date.now()}`,
      metadataUri: "https://example.com/h3paid.json",
    });
    await configureStoreForGame(base, game);

    let err: any = null;
    try {
      await buyGameForBuyer(base, game, null);
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected FreePathNotAllowed").to.not.be.null;
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /FreePathNotAllowed/,
    );
  });

  it("AC-H3-2: free game (is_free=true) + mint_token=None mints with zero settlement", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h3-free-${Date.now()}`,
      metadataUri: "https://example.com/h3free.json",
    });
    const store = await configureStoreForGame(base, game);

    // ANT-483 guard: an ACTIVE option blocks the free flag; deactivate first.
    await deactivatePaymentOptionForGame(base, game, store);

    const gameStoreConfigPda = derivePda(
      [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
      base.storeProgram.programId,
    );
    await base.storeProgram.methods
      .setGameFreeFlag(true)
      .accounts({
        publisher: game.publisher.publicKey,
        game: game.gamePda,
        registryGame: game.registryGamePda,
        gameStoreConfig: gameStoreConfigPda,
      })
      .signers([game.publisher])
      .rpc();

    const { buyer } = await buyGameForBuyer(base, game, null);
    const licensePda = derivePda(
      [Buffer.from("license"), buyer.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );
    const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
    expect(license.holder.toBase58()).to.eq(buyer.publicKey.toBase58());
  });

  it("AC-H3-3: free game + payment fails (no ambiguous middle)", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h3-freepay-${Date.now()}`,
      metadataUri: "https://example.com/h3freepay.json",
    });
    const store = await configureStoreForGame(base, game);

    // ANT-483 guard: an ACTIVE option blocks the free flag; deactivate first.
    await deactivatePaymentOptionForGame(base, game, store);

    const gameStoreConfigPda = derivePda(
      [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
      base.storeProgram.programId,
    );
    await base.storeProgram.methods
      .setGameFreeFlag(true)
      .accounts({
        publisher: game.publisher.publicKey,
        game: game.gamePda,
        registryGame: game.registryGamePda,
        gameStoreConfig: gameStoreConfigPda,
      })
      .signers([game.publisher])
      .rpc();

    let err: any = null;
    try {
      await buyGameForBuyer(base, game, base.paymentMint);
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected FreeGamePaymentNotAllowed").to.not.be.null;
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /FreeGamePaymentNotAllowed/,
    );
  });

  it("AC-H3-4 (ANT-483): set_game_free_flag(true) with an ACTIVE payment option fails", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h3-guard-${Date.now()}`,
      metadataUri: "https://example.com/h3guard.json",
    });
    const store = await configureStoreForGame(base, game);

    // Sanity: one ACTIVE option is tracked on-chain.
    const cfgBefore = (await base.storeProgram.account.gameStoreConfig.fetch(
      store.gameStoreConfigPda,
    )) as any;
    expect(Number(cfgBefore.activePaymentOptions)).to.eq(1);

    let err: any = null;
    try {
      await base.storeProgram.methods
        .setGameFreeFlag(true)
        .accounts({
          publisher: game.publisher.publicKey,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: store.gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected FreeGameHasPaymentOptions").to.not.be.null;
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /FreeGameHasPaymentOptions/,
    );

    // Flag must be unchanged (still paid) after the rejected flip.
    const cfgAfter = (await base.storeProgram.account.gameStoreConfig.fetch(
      store.gameStoreConfigPda,
    )) as any;
    expect(cfgAfter.isFree).to.eq(false);
    expect(Number(cfgAfter.activePaymentOptions)).to.eq(1);
  });

  it("AC-H3-5 (ANT-483): removing the last ACTIVE option unblocks the free flag", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h3-remove-${Date.now()}`,
      metadataUri: "https://example.com/h3remove.json",
    });
    const store = await configureStoreForGame(base, game);

    // Close the ACTIVE option via the remove path (now takes game_store_config).
    await base.storeProgram.methods
      .removeGamePaymentOption()
      .accounts({
        publisher: game.publisher.publicKey,
        authorized_source_program: base.authorizedSourceProgramPda,
        source_program: base.pglProgram.programId,
        game: game.gamePda,
        game_store_config: store.gameStoreConfigPda,
        mint: base.paymentMint,
        game_payment_option: store.gamePaymentOptionPda,
      })
      .signers([game.publisher])
      .rpc();

    const cfgAfterRemove =
      (await base.storeProgram.account.gameStoreConfig.fetch(
        store.gameStoreConfigPda,
      )) as any;
    expect(Number(cfgAfterRemove.activePaymentOptions)).to.eq(0);

    // With no ACTIVE options left, the free flag flip succeeds.
    await base.storeProgram.methods
      .setGameFreeFlag(true)
      .accounts({
        publisher: game.publisher.publicKey,
        game: game.gamePda,
        registryGame: game.registryGamePda,
        gameStoreConfig: store.gameStoreConfigPda,
      })
      .signers([game.publisher])
      .rpc();

    const cfgAfterFlag = (await base.storeProgram.account.gameStoreConfig.fetch(
      store.gameStoreConfigPda,
    )) as any;
    expect(cfgAfterFlag.isFree).to.eq(true);
  });

  it("AC-M1-1: renew_license with neither holder nor actor signing fails MissingRenewalSigner", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `m1-unsigned-${Date.now()}`,
      metadataUri: "https://example.com/m1.json",
    });
    await configureStoreForGame(base, game);
    const { buyer } = await buyGameForBuyer(base, game, base.paymentMint);

    const licensePda = derivePda(
      [Buffer.from("license"), buyer.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );
    const newExpiry = new anchor.BN(Math.floor(Date.now() / 1000) + 172800);
    const outsider = Keypair.generate();

    let err: any = null;
    try {
      // Neither the holder nor the authorized actor signs: an unrelated
      // outsider signs instead (pays nothing; the call must still fail).
      // NOTE: holder/actor are passed as the real keys but without signatures.
      await base.pglProgram.methods
        .renewLicense(newExpiry)
        .accounts({
          actor: base.authority.publicKey,
          holder: buyer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: game.gamePda,
          license: licensePda,
        })
        .signers([outsider])
        .rpc();
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected MissingRenewalSigner").to.not.be.null;
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /MissingRenewalSigner/,
    );
  });

  it("AC-M1-2: holder-signed renewal extends expiry", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `m1-holder-${Date.now()}`,
      metadataUri: "https://example.com/m1holder.json",
    });
    await configureStoreForGame(base, game);
    const { buyer } = await buyGameForBuyer(base, game, base.paymentMint);

    const licensePda = derivePda(
      [Buffer.from("license"), buyer.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );
    const newExpiry = new anchor.BN(Math.floor(Date.now() / 1000) + 172800);

    await base.pglProgram.methods
      .renewLicense(newExpiry)
      .accounts({
        actor: base.authority.publicKey,
        holder: buyer.publicKey,
        authorizedActor: base.storeActorAuthorizedPda,
        game: game.gamePda,
        license: licensePda,
      })
      .signers([buyer])
      .rpc();

    const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
    expect(license.expiresAt.toString()).to.eq(newExpiry.toString());
  });

  it("AC-M5-2: version field present with CURRENT_VERSION=1 (license/receipt/config)", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `m5-ver-${Date.now()}`,
      metadataUri: "https://example.com/m5.json",
    });
    await configureStoreForGame(base, game);
    const { buyer, purchaseReceiptPda } = await buyGameForBuyer(
      base,
      game,
      base.paymentMint,
    );

    const licensePda = derivePda(
      [Buffer.from("license"), buyer.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );
    const gameStoreConfigPda = derivePda(
      [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
      base.storeProgram.programId,
    );
    const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
    const receipt = (await base.storeProgram.account.purchaseReceipt.fetch(
      purchaseReceiptPda,
    )) as any;
    const cfg = (await base.storeProgram.account.gameStoreConfig.fetch(
      gameStoreConfigPda,
    )) as any;
    const regGame = (await base.registryProgram.account.registryGame.fetch(
      game.registryGamePda,
    )) as any;
    expect(Number(license.version)).to.eq(1);
    expect(Number(receipt.version)).to.eq(1);
    expect(Number(cfg.version)).to.eq(1);
    expect(Number(regGame.version)).to.eq(1);
  });
});
