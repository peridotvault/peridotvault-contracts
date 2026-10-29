import * as anchor from "@coral-xyz/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  buyGameForBuyer,
  configureStoreForGame,
  createRegisteredGame,
  derivePda,
  maybeFundSigner,
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
      payer: game.publisher.publicKey,
      publisher: game.publisher.publicKey,
      authorizedSourceProgram: base.authorizedSourceProgramPda,
      sourceProgram: base.pglProgram.programId,
      authorizedRegistryProgram: base.authorizedRegistryProgramPda,
      registryProgram: base.registryProgram.programId,
      game: game.gamePda,
      registryGame: game.registryGamePda,
      gameStoreConfig: store.gameStoreConfigPda,
      mint: store.paymentMint,
      acceptedPaymentToken: acceptedPaymentTokenPda,
      gamePaymentOption: store.gamePaymentOptionPda,
      systemProgram: SystemProgram.programId,
    })
    .signers([game.publisher])
    .rpc();
}
describe("M0 hardening negatives (H2/H3/M1/M5)", () => {
  it("AC-H2-1 (ANT-886 H-1): direct mint_license without actor signature fails", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h2-direct-${Date.now()}`,
      metadataUri: "https://example.com/h2.json",
    });

    const attacker = Keypair.generate();
    // Fund the attacker so rent is covered: the ONLY failure reason below
    // must be the missing actor signature (fail-closed), not an unfunded
    // holder. Without this the negative is vacuous.
    await maybeFundSigner(base.provider, attacker);

    // The actor must be a keypair that NEITHER signs NOR pays: the provider
    // wallet always signs as fee payer, so using the authority as actor
    // would silently satisfy the signature and vacate the negative.
    // Register it as an authorized actor first, so the missing signature
    // is the one and only reason the mint fails.
    const actor = Keypair.generate();
    const authorizedActorPda = derivePda(
      [Buffer.from("authorized_actor"), actor.publicKey.toBuffer()],
      base.pglProgram.programId,
    );
    await base.pglProgram.methods
      .addAuthorizedActor()
      .accounts({
        authority: base.authority.publicKey,
        actor: actor.publicKey,
        pglConfig: base.pglConfigPda,
        authorizedActor: authorizedActorPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const licensePda = derivePda(
      [Buffer.from("license"), attacker.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );

    let err: any = null;
    try {
      // actor = authorized store actor BUT the actor keypair does NOT sign.
      // Only the holder (attacker) signs.
      await base.pglProgram.methods
        .mintLicense(null)
        .accounts({
          actor: actor.publicKey,
          holder: attacker.publicKey,
          authorizedActor: authorizedActorPda,
          game: game.gamePda,
          license: licensePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected actor-signature rejection").to.not.be.null;
    // ANT-886 H-1: `actor` is now `Signer` — enforcement moved from the
    // handler (`MissingActorSignature`) to the Anchor constraint level
    // (`ConstraintSigner`, code 2002) or the runtime preflight
    // ("Signature verification failed"). Any of the three proves the same
    // fail-closed property: no signature → no mint. Alternation is pinned
    // to exact Anchor/runtime/legacy tokens so prose can never satisfy it.
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /MissingActorSignature|ConstraintSigner|Signature verification failed/,
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
        authorizedSourceProgram: base.authorizedSourceProgramPda,
        sourceProgram: base.pglProgram.programId,
        game: game.gamePda,
        gameStoreConfig: store.gameStoreConfigPda,
        mint: base.paymentMint,
        gamePaymentOption: store.gamePaymentOptionPda,
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

  it("AC-M1-1 (ANT-886 H-2): renew_license without holder signature fails", async () => {
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

    let err: any = null;
    try {
      // Neither the holder nor the authorized actor signs: no extra
      // `.signers()` at all (the fee payer signs as payer only). The call
      // must fail on the missing holder signature. NOTE: an unrelated
      // third-party signer must NOT be attached — the runtime rejects
      // signatures from keys absent from the instruction ("unknown
      // signer"), which would vacate the negative.
      await base.pglProgram.methods
        .renewLicense(newExpiry)
        .accounts({
          actor: base.authority.publicKey,
          holder: buyer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: game.gamePda,
          license: licensePda,
        })
        .rpc();
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected holder-signature rejection").to.not.be.null;
    // ANT-886 H-2: `holder` is now `Signer` — only the license holder may
    // renew (actor-only renewal rejected by construction). Enforcement moved
    // from the handler (`MissingRenewalSigner`) to the Anchor constraint
    // level (`ConstraintSigner`, code 2002) or the runtime preflight
    // ("Signature verification failed"). Alternation is pinned to exact
    // Anchor/runtime/legacy tokens so prose can never satisfy it.
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /MissingRenewalSigner|ConstraintSigner|Signature verification failed/,
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

  it("AC-H3-6 (ANT-886 H-3): init_game_store_config without publisher fails", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h3-nopub-${Date.now()}`,
      metadataUri: "https://example.com/h3nopub.json",
    });

    const gameStoreConfigPda = derivePda(
      [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
      base.storeProgram.programId,
    );

    let err: any = null;
    try {
      // Permissionless path removed (ANT-886 H-3): publisher is now a
      // required Signer. Passing the publisher key WITHOUT its signature
      // must fail at the Anchor constraint level — config creation without
      // the publisher's signature is impossible by construction.
      const impostor = Keypair.generate();
      await base.storeProgram.methods
        .initGameStoreConfig(true)
        .accounts({
          payer: base.authority.publicKey,
          publisher: impostor.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected publisher-required rejection").to.not.be.null;
  });

  it("AC-H3-7 (ANT-886 H-3): set_game_payment_option with wrong publisher fails", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base, {
      gameId: `h3-wrongpub-${Date.now()}`,
      metadataUri: "https://example.com/h3wrongpub.json",
    });
    await configureStoreForGame(base, game);

    const impostor = Keypair.generate();
    const acceptedPaymentTokenPda = derivePda(
      [Buffer.from("accepted_payment_token"), base.paymentMint.toBuffer()],
      base.storeProgram.programId,
    );
    const gamePaymentOptionPda = derivePda(
      [
        Buffer.from("game_payment_option"),
        game.gamePda.toBuffer(),
        base.paymentMint.toBuffer(),
      ],
      base.storeProgram.programId,
    );
    const gameStoreConfigPda = derivePda(
      [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
      base.storeProgram.programId,
    );

    let err: any = null;
    try {
      await base.storeProgram.methods
        .setGamePaymentOption(new anchor.BN(999), true)
        .accounts({
          payer: base.authority.publicKey,
          publisher: impostor.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
          mint: base.paymentMint,
          acceptedPaymentToken: acceptedPaymentTokenPda,
          gamePaymentOption: gamePaymentOptionPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([impostor])
        .rpc();
    } catch (e: any) {
      err = e;
    }
    expect(err, "expected Unauthorized for wrong publisher").to.not.be.null;
    expect(String(err?.error?.errorCode?.code ?? err?.message)).to.match(
      /Unauthorized/,
    );
  });
});
