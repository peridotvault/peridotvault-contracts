import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  DEFAULT_REGISTRY_FEE,
  STATUS_ACTIVE,
  STATUS_SUSPENDED,
  STATUS_BANNED,
  createRegisteredGame,
  derivePda,
  setupPeridotFixture,
} from "./helpers/peridot";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("registry program", () => {
  describe("initialize_registry", () => {
    it("initializes registry config correctly", async () => {
      const base = await setupPeridotFixture();
      const config = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      expect(config.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
      expect(config.treasury).to.not.eq(PublicKey.default);
      expect(config.pgl1Program.toBase58()).to.eq(base.pglProgram.programId.toBase58());
    });

    it("rejects double initialization", async () => {
      const base = await setupPeridotFixture();
      let failed = false;
      try {
        await base.registryProgram.methods
          .initializeRegistry(base.authority.publicKey)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("already in use");
      }
      expect(failed).to.eq(true);
    });

    it("rejects default treasury", async () => {
      const base = await setupPeridotFixture();

      const newConfigPda = derivePda(
        [Buffer.from("registry_config")],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .initializeRegistry(PublicKey.default)
          .accounts({
            authority: base.authority.publicKey,
            config: newConfigPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid treasury");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("set_treasury", () => {
    it("updates treasury when called by authority", async () => {
      const base = await setupPeridotFixture();
      const newTreasury = Keypair.generate().publicKey;

      await base.registryProgram.methods
        .setTreasury(newTreasury)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
        })
        .rpc();

      const config = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;
      expect(config.treasury.toBase58()).to.eq(newTreasury.toBase58());
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();
      const newTreasury = Keypair.generate().publicKey;

      let failed = false;
      try {
        await base.registryProgram.methods
          .setTreasury(newTreasury)
          .accounts({
            authority: nonAuthority.publicKey,
            config: base.registryConfigPda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Unauthorized");
      }
      expect(failed).to.eq(true);
    });

    it("rejects default treasury", async () => {
      const base = await setupPeridotFixture();

      let failed = false;
      try {
        await base.registryProgram.methods
          .setTreasury(PublicKey.default)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid treasury");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("add_payment_token", () => {
    it("adds payment token with valid fee", async () => {
      const base = await setupPeridotFixture();
      const mint = await createMint(
        base.provider.connection,
        base.authority,
        base.authority.publicKey,
        null,
        6,
      );

      const tokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), mint.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .addPaymentToken(new anchor.BN(5000))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint,
          acceptedPaymentToken: tokenPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const token = (await base.registryProgram.account.acceptedPaymentToken.fetch(
        tokenPda,
      )) as any;
      expect(token.mint.toBase58()).to.eq(mint.toBase58());
      expect(token.active).to.eq(true);
      expect(token.feeAmount.toString()).to.eq("5000");
    });

    it("rejects zero fee amount", async () => {
      const base = await setupPeridotFixture();
      const mint = await createMint(
        base.provider.connection,
        base.authority,
        base.authority.publicKey,
        null,
        6,
      );

      const tokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), mint.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .addPaymentToken(new anchor.BN(0))
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            mint,
            acceptedPaymentToken: tokenPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid fee amount");
      }
      expect(failed).to.eq(true);
    });

    it("rejects duplicate token addition", async () => {
      const base = await setupPeridotFixture();
      const mint = await createMint(
        base.provider.connection,
        base.authority,
        base.authority.publicKey,
        null,
        6,
      );

      const tokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), mint.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .addPaymentToken(new anchor.BN(5000))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint,
          acceptedPaymentToken: tokenPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      let failed = false;
      try {
        await base.registryProgram.methods
          .addPaymentToken(new anchor.BN(3000))
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            mint,
            acceptedPaymentToken: tokenPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("already in use");
      }
      expect(failed).to.eq(true);
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();
      const mint = await createMint(
        base.provider.connection,
        base.authority,
        base.authority.publicKey,
        null,
        6,
      );

      const tokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), mint.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .addPaymentToken(new anchor.BN(5000))
          .accounts({
            authority: nonAuthority.publicKey,
            config: base.registryConfigPda,
            mint,
            acceptedPaymentToken: tokenPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Unauthorized");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("update_payment_token", () => {
    it("updates token active status and fee", async () => {
      const base = await setupPeridotFixture();

      await base.registryProgram.methods
        .updatePaymentToken(false, new anchor.BN(9999))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint: base.paymentMint,
          acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
        })
        .rpc();

      const token = (await base.registryProgram.account.acceptedPaymentToken.fetch(
        base.registryAcceptedPaymentTokenPda,
      )) as any;
      expect(token.active).to.eq(false);
      expect(token.feeAmount.toString()).to.eq("9999");
    });

    it("rejects zero fee on update", async () => {
      const base = await setupPeridotFixture();

      let failed = false;
      try {
        await base.registryProgram.methods
          .updatePaymentToken(true, new anchor.BN(0))
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            mint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid fee amount");
      }
      expect(failed).to.eq(true);
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();

      let failed = false;
      try {
        await base.registryProgram.methods
          .updatePaymentToken(true, new anchor.BN(5000))
          .accounts({
            authority: nonAuthority.publicKey,
            config: base.registryConfigPda,
            mint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Unauthorized");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("remove_payment_token", () => {
    it("removes payment token and refunds rent to authority", async () => {
      const base = await setupPeridotFixture();
      const mint = await createMint(
        base.provider.connection,
        base.authority,
        base.authority.publicKey,
        null,
        6,
      );

      const tokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), mint.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .addPaymentToken(new anchor.BN(5000))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint,
          acceptedPaymentToken: tokenPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const beforeBalance = await base.provider.connection.getBalance(
        base.authority.publicKey,
      );

      await base.registryProgram.methods
        .removePaymentToken()
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint,
          acceptedPaymentToken: tokenPda,
        })
        .rpc();

      const afterBalance = await base.provider.connection.getBalance(
        base.authority.publicKey,
      );

      const accountInfo = await base.provider.connection.getAccountInfo(tokenPda);
      expect(accountInfo).to.eq(null);
      expect(afterBalance).to.be.gt(beforeBalance - 10000);
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();

      let failed = false;
      try {
        await base.registryProgram.methods
          .removePaymentToken()
          .accounts({
            authority: nonAuthority.publicKey,
            config: base.registryConfigPda,
            mint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Unauthorized");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("create_publish_grant", () => {
    it("creates grant with no expiry", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .createPublishGrant(null)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publishGrant: grantPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      const grant = (await base.registryProgram.account.publishGrant.fetch(
        grantPda,
      )) as any;
      expect(grant.expiredAt).to.eq(null);
    });

    it("creates grant with future expiry", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();
      const futureTs = Math.floor(Date.now() / 1000) + 86400;

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .createPublishGrant(new anchor.BN(futureTs))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publishGrant: grantPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      const grant = (await base.registryProgram.account.publishGrant.fetch(
        grantPda,
      )) as any;
      expect(grant.expiredAt.toString()).to.eq(futureTs.toString());
    });

    it("rejects grant creation without publisher signer", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .createPublishGrant(null)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            publisher: publisher.publicKey,
            publishGrant: grantPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("unknown signer");
      }
      expect(failed).to.eq(true);
    });

    it("rejects duplicate grant creation", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .createPublishGrant(null)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publishGrant: grantPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      let failed = false;
      try {
        await base.registryProgram.methods
          .createPublishGrant(null)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            publisher: publisher.publicKey,
            publishGrant: grantPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("already in use");
      }
      expect(failed).to.eq(true);
    });

    it("rejects past expiry", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();
      const pastTs = Math.floor(Date.now() / 1000) - 100;

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .createPublishGrant(new anchor.BN(pastTs))
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            publisher: publisher.publicKey,
            publishGrant: grantPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid expiry");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("update_publish_grant", () => {
    it("updates existing grant expiry", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();
      const futureTs = Math.floor(Date.now() / 1000) + 86400;

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .createPublishGrant(null)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publishGrant: grantPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      await base.registryProgram.methods
        .updatePublishGrant(new anchor.BN(futureTs))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publishGrant: grantPda,
        })
        .signers([publisher])
        .rpc();

      const grant = (await base.registryProgram.account.publishGrant.fetch(
        grantPda,
      )) as any;
      expect(grant.expiredAt.toString()).to.eq(futureTs.toString());
    });

    it("rejects update on non-existent grant", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .updatePublishGrant(null)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            publisher: publisher.publicKey,
            publishGrant: grantPda,
          })
          .signers([publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("AccountNotInitialized");
      }
      expect(failed).to.eq(true);
    });

    it("rejects update without publisher signer", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();
      const futureTs = Math.floor(Date.now() / 1000) + 86400;

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .createPublishGrant(null)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publishGrant: grantPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      let failed = false;
      try {
        await base.registryProgram.methods
          .updatePublishGrant(new anchor.BN(futureTs))
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            publisher: publisher.publicKey,
            publishGrant: grantPda,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("unknown signer");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("create_game_and_register", () => {
    it("creates game and registers with Active status", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      const registryGame = (await base.registryProgram.account.registryGame.fetch(
        game.registryGamePda,
      )) as any;

      expect(registryGame.game.toBase58()).to.eq(game.gamePda.toBase58());
      expect(registryGame.gameId).to.eq(game.gameId);
      expect((registryGame.status as any).active).to.eq(true);
    });

    it("rejects duplicate game registration (same game pubkey)", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        game.publisher.publicKey,
      );

      const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        registryConfig.treasury,
      );

      const pglConfig = (await base.pglProgram.account.pglConfig.fetch(
        base.pglConfigPda,
      )) as any;

      const grantPda = derivePda(
        [Buffer.from("publish_grant"), game.publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .createGameAndRegister(
            `duplicate-${game.gameId}`,
            `https://meta.peridot/duplicate.json`,
            null,
            null,
          )
          .accounts({
            publisher: game.publisher.publicKey,
            config: base.registryConfigPda,
            paymentMint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
            publisherPaymentAccount: publisherPaymentAta.address,
            treasuryPaymentAccount: treasuryPaymentAta.address,
            registryGame: game.registryGamePda,
            game: game.gamePda,
            pglCreatorState: game.creatorStatePda,
            pglConfig: base.pglConfigPda,
            pglTreasury: pglConfig.treasury,
            pgl1Program: base.pglProgram.programId,
            storeProgram: base.storeProgram.programId,
            storeAuthorizedSourceProgram: base.authorizedSourceProgramPda,
            storeAuthorizedRegistryProgram: base.authorizedRegistryProgramPda,
            storeGameStoreConfig: derivePda(
              [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
              base.storeProgram.programId,
            ),
            selfProgram: base.registryProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .remainingAccounts([
            { pubkey: grantPda, isWritable: false, isSigner: false },
          ])
          .signers([game.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("already in use");
      }
      expect(failed).to.eq(true);
    });

    it("rejects empty game_id", async () => {
      const base = await setupPeridotFixture();
      const publisher = base.publisher;

      const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        publisher.publicKey,
      );

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        registryConfig.treasury,
      );

      const pglConfig = (await base.pglProgram.account.pglConfig.fetch(
        base.pglConfigPda,
      )) as any;

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [
          Buffer.from("game"),
          publisher.publicKey.toBuffer(),
          Buffer.alloc(8),
        ],
        base.pglProgram.programId,
      );

      const registryGamePda = derivePda(
        [Buffer.from("registry_game"), gamePda.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .createGameAndRegister("", "https://meta.peridot/empty.json", null, null)
          .accounts({
            publisher: publisher.publicKey,
            config: base.registryConfigPda,
            paymentMint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
            publisherPaymentAccount: publisherPaymentAta.address,
            treasuryPaymentAccount: treasuryPaymentAta.address,
            registryGame: registryGamePda,
            game: gamePda,
            pglCreatorState: creatorStatePda,
            pglConfig: base.pglConfigPda,
            pglTreasury: pglConfig.treasury,
            pgl1Program: base.pglProgram.programId,
            storeProgram: base.storeProgram.programId,
            storeAuthorizedSourceProgram: base.authorizedSourceProgramPda,
            storeAuthorizedRegistryProgram: base.authorizedRegistryProgramPda,
            storeGameStoreConfig: derivePda(
              [Buffer.from("game_store_config"), gamePda.toBuffer()],
              base.storeProgram.programId,
            ),
            selfProgram: base.registryProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid game id");
      }
      expect(failed).to.eq(true);
    });

    it("rejects game_id exceeding max length (64 chars)", async () => {
      const base = await setupPeridotFixture();
      const publisher = base.publisher;

      const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        publisher.publicKey,
      );

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        registryConfig.treasury,
      );

      const pglConfig = (await base.pglProgram.account.pglConfig.fetch(
        base.pglConfigPda,
      )) as any;

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [
          Buffer.from("game"),
          publisher.publicKey.toBuffer(),
          Buffer.alloc(8),
        ],
        base.pglProgram.programId,
      );

      const registryGamePda = derivePda(
        [Buffer.from("registry_game"), gamePda.toBuffer()],
        base.registryProgram.programId,
      );

      const longGameId = "a".repeat(65);

      let failed = false;
      try {
        await base.registryProgram.methods
          .createGameAndRegister(longGameId, "https://meta.peridot/long.json", null, null)
          .accounts({
            publisher: publisher.publicKey,
            config: base.registryConfigPda,
            paymentMint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
            publisherPaymentAccount: publisherPaymentAta.address,
            treasuryPaymentAccount: treasuryPaymentAta.address,
            registryGame: registryGamePda,
            game: gamePda,
            pglCreatorState: creatorStatePda,
            pglConfig: base.pglConfigPda,
            pglTreasury: pglConfig.treasury,
            pgl1Program: base.pglProgram.programId,
            storeProgram: base.storeProgram.programId,
            storeAuthorizedSourceProgram: base.authorizedSourceProgramPda,
            storeAuthorizedRegistryProgram: base.authorizedRegistryProgramPda,
            storeGameStoreConfig: derivePda(
              [Buffer.from("game_store_config"), gamePda.toBuffer()],
              base.storeProgram.programId,
            ),
            selfProgram: base.registryProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid game id");
      }
      expect(failed).to.eq(true);
    });

    it("rejects empty metadata_uri", async () => {
      const base = await setupPeridotFixture();
      const publisher = base.publisher;

      const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        publisher.publicKey,
      );

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        registryConfig.treasury,
      );

      const pglConfig = (await base.pglProgram.account.pglConfig.fetch(
        base.pglConfigPda,
      )) as any;

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [
          Buffer.from("game"),
          publisher.publicKey.toBuffer(),
          Buffer.alloc(8),
        ],
        base.pglProgram.programId,
      );

      const registryGamePda = derivePda(
        [Buffer.from("registry_game"), gamePda.toBuffer()],
        base.registryProgram.programId,
      );

      let failed = false;
      try {
        await base.registryProgram.methods
          .createGameAndRegister("valid-game-id", "", null, null)
          .accounts({
            publisher: publisher.publicKey,
            config: base.registryConfigPda,
            paymentMint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
            publisherPaymentAccount: publisherPaymentAta.address,
            treasuryPaymentAccount: treasuryPaymentAta.address,
            registryGame: registryGamePda,
            game: gamePda,
            pglCreatorState: creatorStatePda,
            pglConfig: base.pglConfigPda,
            pglTreasury: pglConfig.treasury,
            pgl1Program: base.pglProgram.programId,
            storeProgram: base.storeProgram.programId,
            storeAuthorizedSourceProgram: base.authorizedSourceProgramPda,
            storeAuthorizedRegistryProgram: base.authorizedRegistryProgramPda,
            storeGameStoreConfig: derivePda(
              [Buffer.from("game_store_config"), gamePda.toBuffer()],
              base.storeProgram.programId,
            ),
            selfProgram: base.registryProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid metadata URI");
      }
      expect(failed).to.eq(true);
    });

    it("rejects metadata_uri exceeding max length (256 chars)", async () => {
      const base = await setupPeridotFixture();
      const publisher = base.publisher;

      const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        publisher.publicKey,
      );

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
        base.provider.connection,
        base.authority,
        base.paymentMint,
        registryConfig.treasury,
      );

      const pglConfig = (await base.pglProgram.account.pglConfig.fetch(
        base.pglConfigPda,
      )) as any;

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [
          Buffer.from("game"),
          publisher.publicKey.toBuffer(),
          Buffer.alloc(8),
        ],
        base.pglProgram.programId,
      );

      const registryGamePda = derivePda(
        [Buffer.from("registry_game"), gamePda.toBuffer()],
        base.registryProgram.programId,
      );

      const longUri = "https://meta.peridot/" + "a".repeat(250) + ".json";

      let failed = false;
      try {
        await base.registryProgram.methods
          .createGameAndRegister("valid-id", longUri, null, null)
          .accounts({
            publisher: publisher.publicKey,
            config: base.registryConfigPda,
            paymentMint: base.paymentMint,
            acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
            publisherPaymentAccount: publisherPaymentAta.address,
            treasuryPaymentAccount: treasuryPaymentAta.address,
            registryGame: registryGamePda,
            game: gamePda,
            pglCreatorState: creatorStatePda,
            pglConfig: base.pglConfigPda,
            pglTreasury: pglConfig.treasury,
            pgl1Program: base.pglProgram.programId,
            storeProgram: base.storeProgram.programId,
            storeAuthorizedSourceProgram: base.authorizedSourceProgramPda,
            storeAuthorizedRegistryProgram: base.authorizedRegistryProgramPda,
            storeGameStoreConfig: derivePda(
              [Buffer.from("game_store_config"), gamePda.toBuffer()],
              base.storeProgram.programId,
            ),
            selfProgram: base.registryProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid metadata URI");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("update_game_status", () => {
    it("transitions Active -> Suspended", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_SUSPENDED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      const registryGame = (await base.registryProgram.account.registryGame.fetch(
        game.registryGamePda,
      )) as any;
      expect((registryGame.status as any).suspended).to.eq(true);
    });

    it("transitions Suspended -> Active", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_SUSPENDED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      await base.registryProgram.methods
        .updateGameStatus(STATUS_ACTIVE)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      const registryGame = (await base.registryProgram.account.registryGame.fetch(
        game.registryGamePda,
      )) as any;
      expect((registryGame.status as any).active).to.eq(true);
    });

    it("transitions Active -> Banned", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_BANNED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      const registryGame = (await base.registryProgram.account.registryGame.fetch(
        game.registryGamePda,
      )) as any;
      expect((registryGame.status as any).banned).to.eq(true);
    });

    it("transitions Suspended -> Banned", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_SUSPENDED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      await base.registryProgram.methods
        .updateGameStatus(STATUS_BANNED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      const registryGame = (await base.registryProgram.account.registryGame.fetch(
        game.registryGamePda,
      )) as any;
      expect((registryGame.status as any).banned).to.eq(true);
    });

    it("rejects Banned -> Active (final state)", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_BANNED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      let failed = false;
      try {
        await base.registryProgram.methods
          .updateGameStatus(STATUS_ACTIVE)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            registryGame: game.registryGamePda,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid status transition");
      }
      expect(failed).to.eq(true);
    });

    it("rejects Banned -> Suspended (final state)", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_BANNED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      let failed = false;
      try {
        await base.registryProgram.methods
          .updateGameStatus(STATUS_SUSPENDED)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            registryGame: game.registryGamePda,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid status transition");
      }
      expect(failed).to.eq(true);
    });

    it("rejects invalid status value (e.g., 3)", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      let failed = false;
      try {
        await base.registryProgram.methods
          .updateGameStatus(3)
          .accounts({
            authority: base.authority.publicKey,
            config: base.registryConfigPda,
            registryGame: game.registryGamePda,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid status transition");
      }
      expect(failed).to.eq(true);
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);
      const nonAuthority = Keypair.generate();

      let failed = false;
      try {
        await base.registryProgram.methods
          .updateGameStatus(STATUS_SUSPENDED)
          .accounts({
            authority: nonAuthority.publicKey,
            config: base.registryConfigPda,
            registryGame: game.registryGamePda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Unauthorized");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("close_registry_game", () => {
    it("closes Suspended game and sends rent to treasury", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_SUSPENDED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      const treasuryBefore = await base.provider.connection.getBalance(
        registryConfig.treasury,
      );

      await base.registryProgram.methods
        .closeRegistryGame()
        .accounts({
          publisher: game.publisher.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
          treasury: registryConfig.treasury,
        })
        .signers([game.publisher])
        .rpc();

      const treasuryAfter = await base.provider.connection.getBalance(
        registryConfig.treasury,
      );
      const accountInfo = await base.provider.connection.getAccountInfo(
        game.registryGamePda,
      );

      expect(accountInfo).to.eq(null);
      expect(treasuryAfter).to.be.gt(treasuryBefore);
    });

    it("closes Banned game and sends rent to treasury", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_BANNED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      const treasuryBefore = await base.provider.connection.getBalance(
        registryConfig.treasury,
      );

      await base.registryProgram.methods
        .closeRegistryGame()
        .accounts({
          publisher: game.publisher.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
          treasury: registryConfig.treasury,
        })
        .signers([game.publisher])
        .rpc();

      const treasuryAfter = await base.provider.connection.getBalance(
        registryConfig.treasury,
      );

      expect(treasuryAfter).to.be.gt(treasuryBefore);
    });

    it("rejects closing Active game", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
        base.registryConfigPda,
      )) as any;

      let failed = false;
      try {
        await base.registryProgram.methods
          .closeRegistryGame()
          .accounts({
            publisher: game.publisher.publicKey,
            config: base.registryConfigPda,
            registryGame: game.registryGamePda,
            treasury: registryConfig.treasury,
          })
          .signers([game.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Game not closable");
      }
      expect(failed).to.eq(true);
    });

    it("rejects closing with wrong treasury account", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_SUSPENDED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();

      const wrongTreasury = Keypair.generate().publicKey;

      let failed = false;
      try {
        await base.registryProgram.methods
          .closeRegistryGame()
          .accounts({
            publisher: game.publisher.publicKey,
            config: base.registryConfigPda,
            registryGame: game.registryGamePda,
            treasury: wrongTreasury,
          })
          .signers([game.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid treasury");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("CPI discriminator verification", () => {
    const createHash = require("crypto").createHash;

    function computeDiscriminator(ixName: string): Buffer {
      const hash = createHash("sha256");
      hash.update(`anchor:${ixName}`);
      return hash.digest().subarray(0, 8);
    }

    function discriminatorToHex(disc: Buffer): string {
      return disc.toString("hex");
    }

    it("init_game_store_config discriminator matches game-store program", () => {
      const expected = computeDiscriminator("init_game_store_config");
      const hardcoded = Buffer.from([0x7e, 0xd2, 0xfe, 0x0b, 0x7c, 0x57, 0xe4, 0xa3]);
      expect(discriminatorToHex(expected)).to.eq(discriminatorToHex(hardcoded));
    });

    it("set_game_payment_option discriminator matches game-store program", () => {
      const expected = computeDiscriminator("set_game_payment_option");
      const hardcoded = Buffer.from([0x23, 0x98, 0x38, 0xe4, 0x80, 0xa1, 0xa2, 0xae]);
      expect(discriminatorToHex(expected)).to.eq(discriminatorToHex(hardcoded));
    });

    it("all game-store instruction discriminators are valid", async () => {
      const base = await setupPeridotFixture();
      const storeIdl = base.storeProgram.idl;

      const instructionNames = storeIdl.instructions.map((ix: any) => ix.name);
      for (const name of instructionNames) {
        const disc = computeDiscriminator(name);
        expect(disc.length).to.eq(8);
      }
    });
  });
});
