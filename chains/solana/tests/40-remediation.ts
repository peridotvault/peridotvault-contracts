import { expect } from "chai";
import { getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SystemProgram } from "@solana/web3.js";
import {
  STATUS_ACTIVE,
  STATUS_BANNED,
  STATUS_SUSPENDED,
  createRegisteredGame,
  derivePda,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("remediation regressions", () => {
  it("enforces registry authority and status-transition rules", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

    let unauthorizedFailed = false;
    try {
      await base.registryProgram.methods
        .updateGameStatus(STATUS_SUSPENDED)
        .accounts({
          authority: base.publisher.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .signers([base.publisher])
        .rpc();
    } catch {
      unauthorizedFailed = true;
    }
    expect(unauthorizedFailed).to.eq(true);

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

    let invalidTransitionFailed = false;
    try {
      await base.registryProgram.methods
        .updateGameStatus(STATUS_ACTIVE)
        .accounts({
          authority: base.authority.publicKey,
          config: base.registryConfigPda,
          registryGame: game.registryGamePda,
        })
        .rpc();
    } catch {
      invalidTransitionFailed = true;
    }
    expect(invalidTransitionFailed).to.eq(true);
  });

  it("rejects create_game_and_register with wrong pgl1 program", async () => {
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

    const creatorStatePda = derivePda(
      [Buffer.from("creator_state"), publisher.publicKey.toBuffer()],
      base.pglProgram.programId,
    );
    let nextNonce = BigInt(0);
    try {
      const creatorState = (await base.pglProgram.account.creatorState.fetch(
        creatorStatePda,
      )) as any;
      nextNonce = BigInt(creatorState.nextNonce.toString());
    } catch {
      nextNonce = BigInt(0);
    }

    const nonceBuf = Buffer.alloc(8);
    nonceBuf.writeBigUInt64LE(nextNonce);
    const gamePda = derivePda(
      [Buffer.from("game"), publisher.publicKey.toBuffer(), nonceBuf],
      base.pglProgram.programId,
    );
    const registryGamePda = derivePda(
      [Buffer.from("registry_game"), gamePda.toBuffer()],
      base.registryProgram.programId,
    );
    const publishGrantPda = derivePda(
      [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
      base.registryProgram.programId,
    );
    await base.registryProgram.methods
      .setPublishGrant(null)
      .accounts({
        authority: base.authority.publicKey,
        config: base.registryConfigPda,
        publisher: publisher.publicKey,
        publishGrant: publishGrantPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const pglConfig = (await base.pglProgram.account.pglConfig.fetch(
      base.pglConfigPda,
    )) as any;

    let failed = false;
    try {
      await base.registryProgram.methods
        .createGameAndRegister(`bad-pgl-${Date.now()}`, "https://meta.peridot/bad.json")
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
          // Deliberately wrong program account (must be pgl1)
          pgl1Program: base.registryProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts([
          {
            pubkey: publishGrantPda,
            isWritable: false,
            isSigner: false,
          },
        ])
        .signers([publisher])
        .rpc();
    } catch (error: any) {
      failed = true;
      const err = String(error);
      expect(
        err.includes("Invalid PGL-1 program") ||
          err.includes("InvalidProgramId") ||
          err.includes("caused by account: pgl1_program"),
      ).to.eq(true);
    }

    expect(failed).to.eq(true);
  });
});
