import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  DEFAULT_MAX_REFERRAL_BPS,
  DEFAULT_REFERRAL_BPS,
  DEFAULT_PLATFORM_FEE_BPS,
  setupPeridotFixture,
  derivePda,
  ROLE_SOURCE,
  ROLE_REGISTRY,
} from "./helpers/peridot";

describe("pgl1 program", () => {
  describe("initialize_pgl", () => {
    it("initializes PGL config with valid params", async () => {
      const base = await setupPeridotFixture();
      const pglConfig = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;

      expect(pglConfig.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
      expect(pglConfig.treasury.toBase58()).to.eq(base.authority.publicKey.toBase58());
      expect(pglConfig.createGameFeeLamports.toString()).to.eq("0");
    });
  });

  describe("set_create_game_fee", () => {
    it("updates create game fee", async () => {
      const base = await setupPeridotFixture();
      const newFee = new anchor.BN(50_000_000);

      await base.pglProgram.methods
        .setCreateGameFee(newFee)
        .accounts({
          authority: base.authority.publicKey,
          pglConfig: base.pglConfigPda,
        })
        .rpc();

      const config = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;
      expect(config.createGameFeeLamports.toString()).to.eq(newFee.toString());
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();

      let failed = false;
      try {
        await base.pglProgram.methods
          .setCreateGameFee(new anchor.BN(100))
          .accounts({
            authority: nonAuthority.publicKey,
            pglConfig: base.pglConfigPda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("set_treasury", () => {
    it("updates treasury address", async () => {
      const base = await setupPeridotFixture();
      const newTreasury = Keypair.generate().publicKey;

      await base.pglProgram.methods
        .setTreasury(newTreasury)
        .accounts({
          authority: base.authority.publicKey,
          pglConfig: base.pglConfigPda,
        })
        .rpc();

      const config = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;
      expect(config.treasury.toBase58()).to.eq(newTreasury.toBase58());
    });

    it("rejects default treasury", async () => {
      const base = await setupPeridotFixture();

      let failed = false;
      try {
        await base.pglProgram.methods
          .setTreasury(PublicKey.default)
          .accounts({
            authority: base.authority.publicKey,
            pglConfig: base.pglConfigPda,
          })
          .rpc();
      } catch (error: any) {
        failed = true;
        expect(String(error)).to.include("Invalid treasury");
      }
      expect(failed).to.eq(true);
    });
  });

  describe("set_authority", () => {
    it("transfers authority to new pubkey", async () => {
      const base = await setupPeridotFixture();
      const newAuthority = Keypair.generate().publicKey;

      await base.pglProgram.methods
        .setAuthority(newAuthority)
        .accounts({
          authority: base.authority.publicKey,
          pglConfig: base.pglConfigPda,
        })
        .rpc();

      const config = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;
      expect(config.authority.toBase58()).to.eq(newAuthority.toBase58());
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();

      let failed = false;
      try {
        await base.pglProgram.methods
          .setAuthority(Keypair.generate().publicKey)
          .accounts({
            authority: nonAuthority.publicKey,
            pglConfig: base.pglConfigPda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("authorized_actor", () => {
    it("adds authorized actor", async () => {
      const base = await setupPeridotFixture();
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

      const actorAccount = (await base.pglProgram.account.authorizedActor.fetch(authorizedActorPda)) as any;
      expect(actorAccount.actor.toBase58()).to.eq(actor.publicKey.toBase58());
      expect(actorAccount.active).to.eq(true);
    });

    it("deactivates authorized actor", async () => {
      const base = await setupPeridotFixture();
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

      await base.pglProgram.methods
        .deactivateAuthorizedActor()
        .accounts({
          authority: base.authority.publicKey,
          actor: actor.publicKey,
          pglConfig: base.pglConfigPda,
          authorizedActor: authorizedActorPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const actorAccount = (await base.pglProgram.account.authorizedActor.fetch(authorizedActorPda)) as any;
      expect(actorAccount.active).to.eq(false);
    });

    it("closes authorized actor", async () => {
      const base = await setupPeridotFixture();
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

      const beforeBalance = await base.provider.connection.getBalance(base.authority.publicKey);

      await base.pglProgram.methods
        .closeAuthorizedActor()
        .accounts({
          authority: base.authority.publicKey,
          actor: actor.publicKey,
          pglConfig: base.pglConfigPda,
          authorizedActor: authorizedActorPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const afterBalance = await base.provider.connection.getBalance(base.authority.publicKey);
      expect(afterBalance).to.be.greaterThan(beforeBalance);

      const accountInfo = await base.provider.connection.getAccountInfo(authorizedActorPda);
      expect(accountInfo).to.be.null;
    });

    it("rejects non-authority adding actor", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();
      const actor = Keypair.generate();
      const authorizedActorPda = derivePda(
        [Buffer.from("authorized_actor"), actor.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      let failed = false;
      try {
        await base.pglProgram.methods
          .addAuthorizedActor()
          .accounts({
            authority: nonAuthority.publicKey,
            actor: actor.publicKey,
            pglConfig: base.pglConfigPda,
            authorizedActor: authorizedActorPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("create_game", () => {
    it("creates a game with valid params", async () => {
      const base = await setupPeridotFixture();
      const gameId = `test-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const game = (await base.pglProgram.account.game.fetch(gamePda)) as any;
      expect(game.gameId).to.eq(gameId);
      expect(game.metadataUri).to.eq(metadataUri);
      expect(game.publisher.toBase58()).to.eq(base.publisher.publicKey.toBase58());
      expect(game.nonce.toString()).to.eq("0");
    });

    it("rejects duplicate game_id", async () => {
      const base = await setupPeridotFixture();
      const gameId = `dup-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      let failed = false;
      try {
        await base.pglProgram.methods
          .createGame(gameId, metadataUri)
          .accounts({
            publisher: base.publisher.publicKey,
            creatorState: creatorStatePda,
            pglConfig: base.pglConfigPda,
            game: gamePda,
            systemProgram: SystemProgram.programId,
          })
          .signers([base.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });

    it("rejects game_id too long", async () => {
      const base = await setupPeridotFixture();
      const gameId = "a".repeat(65);
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      let failed = false;
      try {
        await base.pglProgram.methods
          .createGame(gameId, metadataUri)
          .accounts({
            publisher: base.publisher.publicKey,
            creatorState: creatorStatePda,
            pglConfig: base.pglConfigPda,
            game: gamePda,
            systemProgram: SystemProgram.programId,
          })
          .signers([base.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });

    it("rejects metadata_uri too long", async () => {
      const base = await setupPeridotFixture();
      const gameId = `test-${Date.now()}`;
      const metadataUri = "https://example.com/" + "a".repeat(256);

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      let failed = false;
      try {
        await base.pglProgram.methods
          .createGame(gameId, metadataUri)
          .accounts({
            publisher: base.publisher.publicKey,
            creatorState: creatorStatePda,
            pglConfig: base.pglConfigPda,
            game: gamePda,
            systemProgram: SystemProgram.programId,
          })
          .signers([base.publisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("set_publisher", () => {
    it("transfers game publisher to new pubkey", async () => {
      const base = await setupPeridotFixture();
      const newPublisher = Keypair.generate();
      const gameId = `transfer-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      await base.pglProgram.methods
        .setPublisher(newPublisher.publicKey)
        .accounts({
          publisher: base.publisher.publicKey,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const game = (await base.pglProgram.account.game.fetch(gamePda)) as any;
      expect(game.publisher.toBase58()).to.eq(newPublisher.publicKey.toBase58());
    });

    it("rejects non-publisher signer", async () => {
      const base = await setupPeridotFixture();
      const newPublisher = Keypair.generate();
      const gameId = `transfer-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      let failed = false;
      try {
        await base.pglProgram.methods
          .setPublisher(newPublisher.publicKey)
          .accounts({
            publisher: newPublisher.publicKey,
            game: gamePda,
            systemProgram: SystemProgram.programId,
          })
          .signers([newPublisher])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("set_metadata_uri", () => {
    it("updates game metadata URI", async () => {
      const base = await setupPeridotFixture();
      const gameId = `uri-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";
      const newUri = "https://example.com/updated.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      await base.pglProgram.methods
        .setMetadataUri(newUri)
        .accounts({
          publisher: base.publisher.publicKey,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const game = (await base.pglProgram.account.game.fetch(gamePda)) as any;
      expect(game.metadataUri).to.eq(newUri);
    });
  });

  describe("mint_license", () => {
    it("mints license for buyer via authorized actor", async () => {
      const base = await setupPeridotFixture();
      const gameId = `license-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const licensePda = derivePda(
        [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gamePda.toBuffer()],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .mintLicense(null)
        .accounts({
          actor: base.authority.publicKey,
          holder: base.gamer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: gamePda,
          license: licensePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.authority])
        .rpc();

      const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
      expect(license.holder.toBase58()).to.eq(base.gamer.publicKey.toBase58());
      expect(license.game.toBase58()).to.eq(gamePda.toBase58());
      expect(license.expiresAt).to.be.null;
    });

    it("mints license with expiry", async () => {
      const base = await setupPeridotFixture();
      const gameId = `expiry-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";
      const expiresAt = Math.floor(Date.now() / 1000) + 86400;

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const licensePda = derivePda(
        [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gamePda.toBuffer()],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .mintLicense(new anchor.BN(expiresAt))
        .accounts({
          actor: base.authority.publicKey,
          holder: base.gamer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: gamePda,
          license: licensePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.authority])
        .rpc();

      const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
      expect(license.expiresAt.toString()).to.eq(expiresAt.toString());
    });

    it("rejects duplicate license for same holder+game", async () => {
      const base = await setupPeridotFixture();
      const gameId = `dup-license-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const licensePda = derivePda(
        [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gamePda.toBuffer()],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .mintLicense(null)
        .accounts({
          actor: base.authority.publicKey,
          holder: base.gamer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: gamePda,
          license: licensePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.authority])
        .rpc();

      let failed = false;
      try {
        await base.pglProgram.methods
          .mintLicense(null)
          .accounts({
            actor: base.authority.publicKey,
            holder: base.gamer.publicKey,
            authorizedActor: base.storeActorAuthorizedPda,
            game: gamePda,
            license: licensePda,
            systemProgram: SystemProgram.programId,
          })
          .signers([base.authority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });

    it("rejects unauthorized actor", async () => {
      const base = await setupPeridotFixture();
      const gameId = `unauth-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";
      const unauthorizedActor = Keypair.generate();

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const licensePda = derivePda(
        [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gamePda.toBuffer()],
        base.pglProgram.programId,
      );

      const unauthorizedActorPda = derivePda(
        [Buffer.from("authorized_actor"), unauthorizedActor.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      let failed = false;
      try {
        await base.pglProgram.methods
          .mintLicense(null)
          .accounts({
            actor: unauthorizedActor.publicKey,
            holder: base.gamer.publicKey,
            authorizedActor: unauthorizedActorPda,
            game: gamePda,
            license: licensePda,
            systemProgram: SystemProgram.programId,
          })
          .signers([unauthorizedActor])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("renew_license", () => {
    it("renews license expiry", async () => {
      const base = await setupPeridotFixture();
      const gameId = `renew-game-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";
      const initialExpiry = Math.floor(Date.now() / 1000) + 86400;
      const newExpiry = Math.floor(Date.now() / 1000) + 172800;

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const licensePda = derivePda(
        [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gamePda.toBuffer()],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .mintLicense(new anchor.BN(initialExpiry))
        .accounts({
          actor: base.authority.publicKey,
          holder: base.gamer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: gamePda,
          license: licensePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.authority])
        .rpc();

      await base.pglProgram.methods
        .renewLicense(new anchor.BN(newExpiry))
        .accounts({
          actor: base.authority.publicKey,
          holder: base.gamer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: gamePda,
          license: licensePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.authority])
        .rpc();

      const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
      expect(license.expiresAt.toString()).to.eq(newExpiry.toString());
    });

    it("rejects renew on non-existent license", async () => {
      const base = await setupPeridotFixture();
      const gameId = `no-license-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";
      const newExpiry = Math.floor(Date.now() / 1000) + 172800;

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const licensePda = derivePda(
        [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gamePda.toBuffer()],
        base.pglProgram.programId,
      );

      let failed = false;
      try {
        await base.pglProgram.methods
          .renewLicense(new anchor.BN(newExpiry))
          .accounts({
            actor: base.authority.publicKey,
            holder: base.gamer.publicKey,
            authorizedActor: base.storeActorAuthorizedPda,
            game: gamePda,
            license: licensePda,
            systemProgram: SystemProgram.programId,
          })
          .signers([base.authority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("close_creator_state", () => {
    it("closes creator state and refunds lamports", async () => {
      const base = await setupPeridotFixture();
      const gameId = `close-creator-${Date.now()}`;
      const metadataUri = "https://example.com/game.json";

      const creatorStatePda = derivePda(
        [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
        base.pglProgram.programId,
      );

      const gamePda = derivePda(
        [Buffer.from("game"), base.publisher.publicKey.toBuffer(), Buffer.alloc(8)],
        base.pglProgram.programId,
      );

      await base.pglProgram.methods
        .createGame(gameId, metadataUri)
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          game: gamePda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const beforeBalance = await base.provider.connection.getBalance(base.publisher.publicKey);

      await base.pglProgram.methods
        .closeCreatorState()
        .accounts({
          publisher: base.publisher.publicKey,
          creatorState: creatorStatePda,
          pglConfig: base.pglConfigPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([base.publisher])
        .rpc();

      const afterBalance = await base.provider.connection.getBalance(base.publisher.publicKey);
      expect(afterBalance).to.be.greaterThan(beforeBalance);

      const accountInfo = await base.provider.connection.getAccountInfo(creatorStatePda);
      expect(accountInfo).to.be.null;
    });
  });
});
