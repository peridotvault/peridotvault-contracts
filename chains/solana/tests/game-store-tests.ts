import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  setupPeridotFixture,
  derivePda,
  createRegisteredGame,
  configureStoreForGame,
  buyGameForBuyer,
  ROLE_SOURCE,
  ROLE_REGISTRY,
  DEFAULT_GAME_PRICE,
  DEFAULT_PLATFORM_FEE_BPS,
  DEFAULT_REFERRAL_BPS,
  DEFAULT_MAX_REFERRAL_BPS,
  STATUS_ACTIVE,
} from "./helpers/peridot";

describe("game-store program", () => {
  describe("initialize_store", () => {
    it("initializes store config with valid params", async () => {
      const base = await setupPeridotFixture();
      const config = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;

      expect(config.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
      expect(config.treasury.toBase58()).to.eq(base.authority.publicKey.toBase58());
      expect(config.platformFeeBps).to.eq(DEFAULT_PLATFORM_FEE_BPS);
      expect(config.defaultReferralBps).to.eq(DEFAULT_REFERRAL_BPS);
      expect(config.maxReferralBps).to.eq(DEFAULT_MAX_REFERRAL_BPS);
      expect(config.storeActor.toBase58()).to.eq(base.authority.publicKey.toBase58());
    });

    it("rejects default treasury", async () => {
      const base = await setupPeridotFixture();

      let failed = false;
      try {
        await base.storeProgram.methods
          .initializeStore(
            PublicKey.default,
            DEFAULT_PLATFORM_FEE_BPS,
            DEFAULT_REFERRAL_BPS,
            DEFAULT_MAX_REFERRAL_BPS,
            base.authority.publicKey,
          )
          .accounts({
            authority: base.authority.publicKey,
            storeConfig: base.storeConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid treasury");
      }
      expect(failed).to.eq(true);
    });

    it("rejects default store_actor", async () => {
      const base = await setupPeridotFixture();
      const newStoreConfigPda = derivePda(
        [Buffer.from("store_config")],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .initializeStore(
            base.authority.publicKey,
            DEFAULT_PLATFORM_FEE_BPS,
            DEFAULT_REFERRAL_BPS,
            DEFAULT_MAX_REFERRAL_BPS,
            PublicKey.default,
          )
          .accounts({
            authority: base.authority.publicKey,
            storeConfig: newStoreConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid store actor");
      }
      expect(failed).to.eq(true);
    });

    it("rejects platform_fee_bps > MAX", async () => {
      const base = await setupPeridotFixture();
      const newStoreConfigPda = derivePda(
        [Buffer.from("store_config")],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .initializeStore(
            base.authority.publicKey,
            10_001,
            DEFAULT_REFERRAL_BPS,
            DEFAULT_MAX_REFERRAL_BPS,
            base.authority.publicKey,
          )
          .accounts({
            authority: base.authority.publicKey,
            storeConfig: newStoreConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });

    it("rejects max_referral_bps > HARD_CAP", async () => {
      const base = await setupPeridotFixture();
      const newStoreConfigPda = derivePda(
        [Buffer.from("store_config")],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .initializeStore(
            base.authority.publicKey,
            DEFAULT_PLATFORM_FEE_BPS,
            DEFAULT_REFERRAL_BPS,
            5_001,
            base.authority.publicKey,
          )
          .accounts({
            authority: base.authority.publicKey,
            storeConfig: newStoreConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });

    it("rejects default_referral_bps > max_referral_bps", async () => {
      const base = await setupPeridotFixture();
      const newStoreConfigPda = derivePda(
        [Buffer.from("store_config")],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .initializeStore(
            base.authority.publicKey,
            DEFAULT_PLATFORM_FEE_BPS,
            6_000,
            5_000,
            base.authority.publicKey,
          )
          .accounts({
            authority: base.authority.publicKey,
            storeConfig: newStoreConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("store admin config", () => {
    it("set_treasury updates treasury", async () => {
      const base = await setupPeridotFixture();
      const newTreasury = Keypair.generate().publicKey;

      await base.storeProgram.methods
        .setTreasury(newTreasury)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
        })
        .rpc();

      const config = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;
      expect(config.treasury.toBase58()).to.eq(newTreasury.toBase58());
    });

    it("set_platform_fee updates fee", async () => {
      const base = await setupPeridotFixture();

      await base.storeProgram.methods
        .setPlatformFee(2_000)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
        })
        .rpc();

      const config = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;
      expect(config.platformFeeBps).to.eq(2_000);
    });

    it("set_default_referral updates default", async () => {
      const base = await setupPeridotFixture();

      await base.storeProgram.methods
        .setDefaultReferral(500)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
        })
        .rpc();

      const config = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;
      expect(config.defaultReferralBps).to.eq(500);
    });

    it("set_max_referral updates max", async () => {
      const base = await setupPeridotFixture();

      await base.storeProgram.methods
        .setMaxReferral(3_000)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
        })
        .rpc();

      const config = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;
      expect(config.maxReferralBps).to.eq(3_000);
    });

    it("set_store_actor updates actor", async () => {
      const base = await setupPeridotFixture();
      const newActor = Keypair.generate().publicKey;

      await base.storeProgram.methods
        .setStoreActor(newActor)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
        })
        .rpc();

      const config = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;
      expect(config.storeActor.toBase58()).to.eq(newActor.toBase58());
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();

      let failed = false;
      try {
        await base.storeProgram.methods
          .setTreasury(Keypair.generate().publicKey)
          .accounts({
            authority: nonAuthority.publicKey,
            storeConfig: base.storeConfigPda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("authorized_program", () => {
    it("add_authorized_program with role=0 (source)", async () => {
      const base = await setupPeridotFixture();
      const newProgram = Keypair.generate();
      const authorizedPda = derivePda(
        [Buffer.from("authorized_program"), newProgram.publicKey.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .addAuthorizedProgram(ROLE_SOURCE)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
          programId: newProgram.publicKey,
          authorizedProgram: authorizedPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const program = (await base.storeProgram.account.authorizedProgram.fetch(authorizedPda)) as any;
      expect(program.programId.toBase58()).to.eq(newProgram.publicKey.toBase58());
      expect(program.active).to.eq(true);
      expect(program.role).to.eq(ROLE_SOURCE);
    });

    it("add_authorized_program with role=1 (registry)", async () => {
      const base = await setupPeridotFixture();
      const newProgram = Keypair.generate();
      const authorizedPda = derivePda(
        [Buffer.from("authorized_program"), newProgram.publicKey.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .addAuthorizedProgram(ROLE_REGISTRY)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
          programId: newProgram.publicKey,
          authorizedProgram: authorizedPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const program = (await base.storeProgram.account.authorizedProgram.fetch(authorizedPda)) as any;
      expect(program.role).to.eq(ROLE_REGISTRY);
    });

    it("update_authorized_program changes active and role", async () => {
      const base = await setupPeridotFixture();
      const newProgram = Keypair.generate();
      const authorizedPda = derivePda(
        [Buffer.from("authorized_program"), newProgram.publicKey.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .addAuthorizedProgram(ROLE_SOURCE)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
          programId: newProgram.publicKey,
          authorizedProgram: authorizedPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      await base.storeProgram.methods
        .updateAuthorizedProgram(false, ROLE_REGISTRY)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
          authorizedProgram: authorizedPda,
        })
        .rpc();

      const program = (await base.storeProgram.account.authorizedProgram.fetch(authorizedPda)) as any;
      expect(program.active).to.eq(false);
      expect(program.role).to.eq(ROLE_REGISTRY);
    });

    it("rejects invalid role > ROLE_REGISTRY", async () => {
      const base = await setupPeridotFixture();
      const newProgram = Keypair.generate();
      const authorizedPda = derivePda(
        [Buffer.from("authorized_program"), newProgram.publicKey.toBuffer()],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .addAuthorizedProgram(5)
          .accounts({
            authority: base.authority.publicKey,
            storeConfig: base.storeConfigPda,
            programId: newProgram.publicKey,
            authorizedProgram: authorizedPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid program role");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("payment_token", () => {
    it("add_payment_token creates token entry", async () => {
      const base = await setupPeridotFixture();
      const newMint = Keypair.generate();
      const acceptedPda = derivePda(
        [Buffer.from("accepted_payment_token"), newMint.publicKey.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .addPaymentToken()
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
          mint: newMint.publicKey,
          acceptedPaymentToken: acceptedPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const token = (await base.storeProgram.account.acceptedPaymentToken.fetch(acceptedPda)) as any;
      expect(token.mint.toBase58()).to.eq(newMint.publicKey.toBase58());
      expect(token.active).to.eq(true);
    });

    it("update_payment_token changes active status", async () => {
      const base = await setupPeridotFixture();
      const newMint = Keypair.generate();
      const acceptedPda = derivePda(
        [Buffer.from("accepted_payment_token"), newMint.publicKey.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .addPaymentToken()
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
          mint: newMint.publicKey,
          acceptedPaymentToken: acceptedPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      await base.storeProgram.methods
        .updatePaymentToken(false)
        .accounts({
          authority: base.authority.publicKey,
          storeConfig: base.storeConfigPda,
          acceptedPaymentToken: acceptedPda,
        })
        .rpc();

      const token = (await base.storeProgram.account.acceptedPaymentToken.fetch(acceptedPda)) as any;
      expect(token.active).to.eq(false);
    });
  });

  describe("game_store_config", () => {
    it("init_game_store_config by publisher", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .initGameStoreConfig(true)
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([game.publisher])
        .rpc();

      const config = (await base.storeProgram.account.gameStoreConfig.fetch(gameStoreConfigPda)) as any;
      expect(config.game.toBase58()).to.eq(game.gamePda.toBase58());
      expect(config.active).to.eq(true);
      expect(config.referralBps).to.be.null;
      expect(config.discountBps).to.be.null;
    });

    it("set_game_store_active toggles active flag", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .initGameStoreConfig(true)
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([game.publisher])
        .rpc();

      await base.storeProgram.methods
        .setGameStoreActive(false)
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([game.publisher])
        .rpc();

      const config = (await base.storeProgram.account.gameStoreConfig.fetch(gameStoreConfigPda)) as any;
      expect(config.active).to.eq(false);
    });

    it("rejects non-publisher signer", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );
      const nonPublisher = Keypair.generate();

      let failed = false;
      try {
        await base.storeProgram.methods
          .initGameStoreConfig(true)
          .accounts({
            publisher: nonPublisher.publicKey,
            authorizedSourceProgram: base.authorizedSourceProgramPda,
            sourceProgram: base.pglProgram.programId,
            authorizedRegistryProgram: base.authorizedRegistryProgramPda,
            registryProgram: base.registryProgram.programId,
            game: game.gamePda,
            registryGame: game.registryGamePda,
            gameStoreConfig: gameStoreConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([nonPublisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("game_payment_option", () => {
    it("set_game_payment_option creates payment option", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      const storeFixture = await configureStoreForGame(base, game);

      const option = (await base.storeProgram.account.gamePaymentOption.fetch(storeFixture.gamePaymentOptionPda)) as any;
      expect(option.game.toBase58()).to.eq(game.gamePda.toBase58());
      expect(option.mint.toBase58()).to.eq(base.paymentMint.toBase58());
      expect(option.basePrice.toString()).to.eq(DEFAULT_GAME_PRICE.toString());
      expect(option.active).to.eq(true);
    });

    it("remove_game_payment_option closes PDA", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      const storeFixture = await configureStoreForGame(base, game);

      const beforeBalance = await base.provider.connection.getBalance(game.publisher.publicKey);

      await base.storeProgram.methods
        .removeGamePaymentOption()
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          game: game.gamePda,
          mint: base.paymentMint,
          gamePaymentOption: storeFixture.gamePaymentOptionPda,
        })
        .signers([game.publisher])
        .rpc();

      const afterBalance = await base.provider.connection.getBalance(game.publisher.publicKey);
      expect(afterBalance).to.be.greaterThan(beforeBalance);

      const accountInfo = await base.provider.connection.getAccountInfo(storeFixture.gamePaymentOptionPda);
      expect(accountInfo).to.be.null;
    });
  });

  describe("discount", () => {
    it("set_discount applies discount", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      const discountBps = 2_000;
      const startsAt = Math.floor(Date.now() / 1000) - 3600;
      const expiresAt = Math.floor(Date.now() / 1000) + 86400;

      await base.storeProgram.methods
        .setDiscount(new anchor.BN(discountBps), new anchor.BN(startsAt), new anchor.BN(expiresAt))
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      const config = (await base.storeProgram.account.gameStoreConfig.fetch(gameStoreConfigPda)) as any;
      expect(config.discountBps).to.eq(discountBps);
      expect(config.discountStartsAt.toString()).to.eq(startsAt.toString());
      expect(config.discountExpiresAt.toString()).to.eq(expiresAt.toString());
    });

    it("clear_discount resets discount fields", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .setDiscount(new anchor.BN(1_000), new anchor.BN(0), new anchor.BN(9999999999))
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      await base.storeProgram.methods
        .clearDiscount()
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          game: game.gamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      const config = (await base.storeProgram.account.gameStoreConfig.fetch(gameStoreConfigPda)) as any;
      expect(config.discountBps).to.be.null;
      expect(config.discountStartsAt).to.be.null;
      expect(config.discountExpiresAt).to.be.null;
    });

    it("rejects discount_bps > 10_000", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .setDiscount(new anchor.BN(10_001), new anchor.BN(0), new anchor.BN(9999999999))
          .accounts({
            publisher: game.publisher.publicKey,
            authorizedSourceProgram: base.authorizedSourceProgramPda,
            sourceProgram: base.pglProgram.programId,
            authorizedRegistryProgram: base.authorizedRegistryProgramPda,
            registryProgram: base.registryProgram.programId,
            game: game.gamePda,
            registryGame: game.registryGamePda,
            gameStoreConfig: gameStoreConfigPda,
          })
          .signers([game.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });

    it("rejects discount start >= end", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .setDiscount(new anchor.BN(1_000), new anchor.BN(100), new anchor.BN(50))
          .accounts({
            publisher: game.publisher.publicKey,
            authorizedSourceProgram: base.authorizedSourceProgramPda,
            sourceProgram: base.pglProgram.programId,
            authorizedRegistryProgram: base.authorizedRegistryProgramPda,
            registryProgram: base.registryProgram.programId,
            game: game.gamePda,
            registryGame: game.registryGamePda,
            gameStoreConfig: gameStoreConfigPda,
          })
          .signers([game.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("referral_bps", () => {
    it("set_referral_bps sets game-specific referral", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .setReferralBps(new anchor.BN(1_000))
        .accounts({
          publisher: game.publisher.publicKey,
          storeConfig: base.storeConfigPda,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          game: game.gamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      const config = (await base.storeProgram.account.gameStoreConfig.fetch(gameStoreConfigPda)) as any;
      expect(config.referralBps).to.eq(1_000);
    });

    it("set_referral_bps normalizes Some(0) to None", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .setReferralBps(new anchor.BN(1_000))
        .accounts({
          publisher: game.publisher.publicKey,
          storeConfig: base.storeConfigPda,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          game: game.gamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      await base.storeProgram.methods
        .setReferralBps(new anchor.BN(0))
        .accounts({
          publisher: game.publisher.publicKey,
          storeConfig: base.storeConfigPda,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          game: game.gamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      const config = (await base.storeProgram.account.gameStoreConfig.fetch(gameStoreConfigPda)) as any;
      expect(config.referralBps).to.be.null;
    });

    it("rejects referral_bps > max_referral_bps", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      let failed = false;
      try {
        await base.storeProgram.methods
          .setReferralBps(new anchor.BN(6_000))
          .accounts({
            publisher: game.publisher.publicKey,
            storeConfig: base.storeConfigPda,
            authorizedSourceProgram: base.authorizedSourceProgramPda,
            sourceProgram: base.pglProgram.programId,
            game: game.gamePda,
            gameStoreConfig: gameStoreConfigPda,
          })
          .signers([game.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("buy_game", () => {
    it("buys game successfully", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const { buyer, purchaseReceiptPda } = await buyGameForBuyer(base, game, base.paymentMint);

      const receipt = (await base.storeProgram.account.purchaseReceipt.fetch(purchaseReceiptPda)) as any;
      expect(receipt.buyer.toBase58()).to.eq(buyer.publicKey.toBase58());
      expect(receipt.game.toBase58()).to.eq(game.gamePda.toBase58());
      expect(receipt.paymentMint.toBase58()).to.eq(base.paymentMint.toBase58());
      expect(receipt.paidAmount.toString()).to.eq(DEFAULT_GAME_PRICE.toString());
      expect(receipt.finalPrice.toString()).to.eq(DEFAULT_GAME_PRICE.toString());
      expect(receipt.referrer.toBase58()).to.eq(PublicKey.default.toBase58());
      expect(receipt.referralBpsApplied).to.eq(0);
    });

    it("rejects duplicate purchase (already owned)", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      await buyGameForBuyer(base, game, base.paymentMint);

      let failed = false;
      try {
        await buyGameForBuyer(base, game, base.paymentMint);
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });

    it("rejects purchase when game inactive in store", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game, { active: false });

      let failed = false;
      try {
        await buyGameForBuyer(base, game, base.paymentMint);
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Game not active in store");
      }
      expect(failed).to.eq(true);
    });

    it("rejects purchase when payment option inactive", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game, { active: true });

      const gamePaymentOptionPda = derivePda(
        [Buffer.from("game_payment_option"), game.gamePda.toBuffer(), base.paymentMint.toBuffer()],
        base.storeProgram.programId,
      );

      await base.storeProgram.methods
        .setGamePaymentOption(new anchor.BN(DEFAULT_GAME_PRICE), false)
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: derivePda(
            [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
            base.storeProgram.programId,
          ),
          mint: base.paymentMint,
          acceptedPaymentToken: base.storeAcceptedPaymentTokenPda,
          gamePaymentOption: gamePaymentOptionPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([game.publisher])
        .rpc();

      let failed = false;
      try {
        await buyGameForBuyer(base, game, base.paymentMint);
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Price not found");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("buy_game with referral", () => {
    it("applies referral commission", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const referrer = Keypair.generate();
      const referrerAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        referrer.publicKey,
      );

      const buyerAtaBefore = await base.provider.connection.getTokenAccountBalance(
        referrerAta.address,
      );

      await buyGameForBuyer(base, game, base.paymentMint, { referrer: referrer.publicKey });

      const buyerAtaAfter = await base.provider.connection.getTokenAccountBalance(
        referrerAta.address,
      );

      const referralReceived = parseInt(buyerAtaAfter.value.amount) - parseInt(buyerAtaBefore.value.amount);
      const expectedReferral = Math.floor(DEFAULT_GAME_PRICE * DEFAULT_REFERRAL_BPS / 10_000);
      expect(referralReceived).to.eq(expectedReferral);
    });

    it("stores referrer in purchase receipt", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const referrer = Keypair.generate();
      const { purchaseReceiptPda } = await buyGameForBuyer(base, game, base.paymentMint, {
        referrer: referrer.publicKey,
      });

      const receipt = (await base.storeProgram.account.purchaseReceipt.fetch(purchaseReceiptPda)) as any;
      expect(receipt.referrer.toBase58()).to.eq(referrer.publicKey.toBase58());
      expect(receipt.referralBpsApplied).to.eq(DEFAULT_REFERRAL_BPS);
    });
  });

  describe("buy_game with discount", () => {
    it("applies discount to final price", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      const discountBps = 2_000;
      const startsAt = Math.floor(Date.now() / 1000) - 3600;
      const expiresAt = Math.floor(Date.now() / 1000) + 86400;

      await base.storeProgram.methods
        .setDiscount(new anchor.BN(discountBps), new anchor.BN(startsAt), new anchor.BN(expiresAt))
        .accounts({
          publisher: game.publisher.publicKey,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          authorizedRegistryProgram: base.authorizedRegistryProgramPda,
          registryProgram: base.registryProgram.programId,
          game: game.gamePda,
          registryGame: game.registryGamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      const expectedFinalPrice = DEFAULT_GAME_PRICE - Math.floor(DEFAULT_GAME_PRICE * discountBps / 10_000);

      const { purchaseReceiptPda } = await buyGameForBuyer(base, game, base.paymentMint);

      const receipt = (await base.storeProgram.account.purchaseReceipt.fetch(purchaseReceiptPda)) as any;
      expect(receipt.finalPrice.toString()).to.eq(expectedFinalPrice.toString());
      expect(receipt.paidAmount.toString()).to.eq(expectedFinalPrice.toString());
    });
  });

  describe("buy_game with game-specific referral", () => {
    it("uses game-specific referral bps over default", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      await configureStoreForGame(base, game);

      const gameStoreConfigPda = derivePda(
        [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
        base.storeProgram.programId,
      );

      const customReferralBps = 1_500;
      await base.storeProgram.methods
        .setReferralBps(new anchor.BN(customReferralBps))
        .accounts({
          publisher: game.publisher.publicKey,
          storeConfig: base.storeConfigPda,
          authorizedSourceProgram: base.authorizedSourceProgramPda,
          sourceProgram: base.pglProgram.programId,
          game: game.gamePda,
          gameStoreConfig: gameStoreConfigPda,
        })
        .signers([game.publisher])
        .rpc();

      const referrer = Keypair.generate();
      const referrerAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        referrer.publicKey,
      );

      const buyerAtaBefore = await base.provider.connection.getTokenAccountBalance(
        referrerAta.address,
      );

      await buyGameForBuyer(base, game, base.paymentMint, { referrer: referrer.publicKey });

      const buyerAtaAfter = await base.provider.connection.getTokenAccountBalance(
        referrerAta.address,
      );

      const referralReceived = parseInt(buyerAtaAfter.value.amount) - parseInt(buyerAtaBefore.value.amount);
      const expectedReferral = Math.floor(DEFAULT_GAME_PRICE * customReferralBps / 10_000);
      expect(referralReceived).to.eq(expectedReferral);
    });
  });
});
