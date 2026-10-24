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
    await configureStoreForGame(base, game);

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
    await configureStoreForGame(base, game);

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
