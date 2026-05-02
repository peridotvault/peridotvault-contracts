import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  setupPeridotFixture,
  derivePda,
  createRegisteredGame,
  ROLE_SOURCE,
  ROLE_REGISTRY,
  STATUS_ACTIVE,
  STATUS_SUSPENDED,
  STATUS_BANNED,
} from "./helpers/peridot";

describe("registry program", () => {
  describe("initialize_registry", () => {
    it("initializes registry config with valid params", async () => {
      const base = await setupPeridotFixture();
      const config = (await base.registryProgram.account.registryConfig.fetch(base.registryConfigPda)) as any;

      expect(config.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
      expect(config.treasury.toBase58()).to.eq(base.authority.publicKey.toBase58());
      expect(config.pgl1Program.toBase58()).to.eq(base.pglProgram.programId.toBase58());
    });
  });

  describe("set_treasury", () => {
    it("updates treasury address", async () => {
      const base = await setupPeridotFixture();
      const newTreasury = Keypair.generate().publicKey;

      await base.registryProgram.methods
        .setTreasury(newTreasury)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
        })
        .rpc();

      const config = (await base.registryProgram.account.registryConfig.fetch(base.registryConfigPda)) as any;
      expect(config.treasury.toBase58()).to.eq(newTreasury.toBase58());
    });

    it("rejects non-authority signer", async () => {
      const base = await setupPeridotFixture();
      const nonAuthority = Keypair.generate();

      let failed = false;
      try {
        await base.registryProgram.methods
          .setTreasury(Keypair.generate().publicKey)
          .accounts({
            authority: nonAuthority.publicKey,
            config: base.registryConfigPda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("payment_token", () => {
    it("adds payment token with fee", async () => {
      const base = await setupPeridotFixture();
      const newMint = Keypair.generate();
      const feeAmount = new anchor.BN(500);

      const acceptedTokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), newMint.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .addPaymentToken(feeAmount)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint: newMint.publicKey,
          accepted_payment_token: acceptedTokenPda,
          system_program: SystemProgram.programId,
        })
        .rpc();

      const token = (await base.registryProgram.account.acceptedPaymentToken.fetch(acceptedTokenPda)) as any;
      expect(token.mint.toBase58()).to.eq(newMint.publicKey.toBase58());
      expect(token.active).to.eq(true);
      expect(token.feeAmount.toString()).to.eq(feeAmount.toString());
    });

    it("updates payment token active status and fee", async () => {
      const base = await setupPeridotFixture();
      const newMint = Keypair.generate();
      const feeAmount = new anchor.BN(500);

      const acceptedTokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), newMint.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .addPaymentToken(feeAmount)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint: newMint.publicKey,
          accepted_payment_token: acceptedTokenPda,
          system_program: SystemProgram.programId,
        })
        .rpc();

      const newFee = new anchor.BN(1000);
      await base.registryProgram.methods
        .updatePaymentToken(false, newFee)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          accepted_payment_token: acceptedTokenPda,
        })
        .rpc();

      const token = (await base.registryProgram.account.acceptedPaymentToken.fetch(acceptedTokenPda)) as any;
      expect(token.active).to.eq(false);
      expect(token.feeAmount.toString()).to.eq(newFee.toString());
    });

    it("removes payment token", async () => {
      const base = await setupPeridotFixture();
      const newMint = Keypair.generate();
      const feeAmount = new anchor.BN(500);

      const acceptedTokenPda = derivePda(
        [Buffer.from("accepted_payment_token"), newMint.publicKey.toBuffer()],
        base.registryProgram.programId,
      );

      await base.registryProgram.methods
        .addPaymentToken(feeAmount)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          mint: newMint.publicKey,
          accepted_payment_token: acceptedTokenPda,
          system_program: SystemProgram.programId,
        })
        .rpc();

      const beforeBalance = await base.provider.connection.getBalance(base.authority.publicKey);

      await base.registryProgram.methods
        .removePaymentToken()
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          accepted_payment_token: acceptedTokenPda,
        })
        .rpc();

      const afterBalance = await base.provider.connection.getBalance(base.authority.publicKey);
      expect(afterBalance).to.be.greaterThan(beforeBalance);

      const accountInfo = await base.provider.connection.getAccountInfo(acceptedTokenPda);
      expect(accountInfo).to.be.null;
    });
  });

  describe("publish_grant", () => {
    it("creates publish grant without expiry", async () => {
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
          publish_grant: grantPda,
          system_program: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      const grant = (await base.registryProgram.account.publishGrant.fetch(grantPda)) as any;
      expect(grant.expiredAt).to.be.null;
    });

    it("creates publish grant with expiry", async () => {
      const base = await setupPeridotFixture();
      const publisher = Keypair.generate();
      const grantPda = derivePda(
        [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
        base.registryProgram.programId,
      );
      const expiredAt = Math.floor(Date.now() / 1000) + 86400;

      await base.registryProgram.methods
        .createPublishGrant(new anchor.BN(expiredAt))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publish_grant: grantPda,
          system_program: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      const grant = (await base.registryProgram.account.publishGrant.fetch(grantPda)) as any;
      expect(grant.expiredAt.toString()).to.eq(expiredAt.toString());
    });

    it("updates publish grant expiry", async () => {
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
          publish_grant: grantPda,
          system_program: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      const newExpiry = Math.floor(Date.now() / 1000) + 172800;
      await base.registryProgram.methods
        .updatePublishGrant(new anchor.BN(newExpiry))
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          publisher: publisher.publicKey,
          publish_grant: grantPda,
          system_program: SystemProgram.programId,
        })
        .signers([publisher])
        .rpc();

      const grant = (await base.registryProgram.account.publishGrant.fetch(grantPda)) as any;
      expect(grant.expiredAt.toString()).to.eq(newExpiry.toString());
    });
  });

  describe("create_game_and_register", () => {
    it("creates game in PGL1 and registers in registry", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      const registryGame = (await base.registryProgram.account.registryGame.fetch(game.registryGamePda)) as any;
      expect(registryGame.gameId).to.eq(game.gameId);
      expect(registryGame.game.toBase58()).to.eq(game.gamePda.toBase58());
      expect(registryGame.status).to.eq(STATUS_ACTIVE);

      const pglGame = (await base.pglProgram.account.game.fetch(game.gamePda)) as any;
      expect(pglGame.gameId).to.eq(game.gameId);
      expect(pglGame.publisher.toBase58()).to.eq(game.publisher.publicKey.toBase58());
    });

    it("rejects duplicate game_id", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      let failed = false;
      try {
        await createRegisteredGame(base, { gameId: game.gameId });
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("update_game_status", () => {
    it("updates game status to suspended", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_SUSPENDED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registry_game: game.registryGamePda,
        })
        .rpc();

      const registryGame = (await base.registryProgram.account.registryGame.fetch(game.registryGamePda)) as any;
      expect(registryGame.status).to.eq(STATUS_SUSPENDED);
    });

    it("updates game status to banned", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      await base.registryProgram.methods
        .updateGameStatus(STATUS_BANNED)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registry_game: game.registryGamePda,
        })
        .rpc();

      const registryGame = (await base.registryProgram.account.registryGame.fetch(game.registryGamePda)) as any;
      expect(registryGame.status).to.eq(STATUS_BANNED);
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
            registry_game: game.registryGamePda,
          })
          .signers([nonAuthority])
          .rpc();
      } catch (error: any) {
        failed = true;
      }
      expect(failed).to.eq(true);
    });
  });

  describe("close_registry_game", () => {
    it("closes registry game and refunds lamports", async () => {
      const base = await setupPeridotFixture();
      const game = await createRegisteredGame(base);

      const beforeBalance = await base.provider.connection.getBalance(base.authority.publicKey);

      await base.registryProgram.methods
        .closeRegistryGame()
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registry_game: game.registryGamePda,
        })
        .rpc();

      const afterBalance = await base.provider.connection.getBalance(base.authority.publicKey);
      expect(afterBalance).to.be.greaterThan(beforeBalance);

      const accountInfo = await base.provider.connection.getAccountInfo(game.registryGamePda);
      expect(accountInfo).to.be.null;
    });
  });
});
