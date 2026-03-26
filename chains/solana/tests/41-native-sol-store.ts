import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { getAssociatedTokenAddressSync, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, SystemProgram } from "@solana/web3.js";

import {
  GAME_AUTHORITY_SEED,
  LICENSE_SEED,
  MINTER_AUTH_SEED,
  TEST_GAME_ID,
  GAME_REGISTRATION_SEED,
  ensureGameCreated,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("native SOL store flow", () => {
  it("supports pricing, buying, and publisher withdrawal in SOL", async () => {
    const base = await setupPeridotFixture();
    const game = await ensureGameCreated(base);

    const [gameRegistrationPda] = PublicKey.findProgramAddressSync(
      [GAME_REGISTRATION_SEED, Buffer.from(TEST_GAME_ID)],
      base.registryProgram.programId,
    );

    await base.registryProgram.methods
      .setStatus(TEST_GAME_ID, 1)
      .accounts({
        admin: base.governance.publicKey,
        registryState: base.registryStatePda,
        gameRegistration: gameRegistrationPda,
      } as any)
      .signers([base.governance])
      .rpc();

    await base.storeProgram.methods
      .setPrice(TEST_GAME_ID, new anchor.BN(100_000_000), SystemProgram.programId)
      .accounts({
        publisher: base.publisher.publicKey,
        storeState: base.storeStatePda,
        registryState: base.registryStatePda,
        pgcGameState: game.gameStatePda,
        gameRegistration: gameRegistrationPda,
      } as any)
      .signers([base.publisher])
      .rpc();

    await base.storeProgram.methods
      .setDiscount(TEST_GAME_ID, 2_000)
      .accounts({
        publisher: base.publisher.publicKey,
        storeState: base.storeStatePda,
        registryState: base.registryStatePda,
        pgcGameState: game.gameStatePda,
        gameRegistration: gameRegistrationPda,
      } as any)
      .signers([base.publisher])
      .rpc();

    const [storeMinterAuth] = PublicKey.findProgramAddressSync(
      [MINTER_AUTH_SEED, game.gameStatePda.toBuffer(), base.storeStatePda.toBuffer()],
      base.pgcProgram.programId,
    );
    const [licensePda] = PublicKey.findProgramAddressSync(
      [LICENSE_SEED, game.gameStatePda.toBuffer(), base.gamer.publicKey.toBuffer()],
      base.pgcProgram.programId,
    );
    const userGameTokenAccount = getAssociatedTokenAddressSync(
      game.mintPda,
      base.gamer.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    const treasuryLamportsBefore = await base.provider.connection.getBalance(
      base.treasury.publicKey,
    );
    const publisherLamportsBefore = await base.provider.connection.getBalance(
      base.publisher.publicKey,
    );

    await base.storeProgram.methods
      .buyGame(TEST_GAME_ID)
      .accounts({
        buyer: base.gamer.publicKey,
        storeState: base.storeStatePda,
        registryState: base.registryStatePda,
        pgcProgram: base.pgcProgram.programId,
        pgcGameState: game.gameStatePda,
        gameAuthority: game.gameAuthorityPda,
        storeMinterAuth,
        licenseAccount: licensePda,
        userGameTokenAccount,
        gameMint: game.mintPda,
        treasury: base.treasury.publicKey,
        licenseTokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
        systemProgram: SystemProgram.programId,
        gameRegistration: gameRegistrationPda,
      } as any)
      .signers([base.gamer])
      .rpc();

    const treasuryLamportsAfter = await base.provider.connection.getBalance(
      base.treasury.publicKey,
    );
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const publisherBalance = storeState.publisherBalances.find(
      (entry: any) =>
        entry.publisher.toBase58() === base.publisher.publicKey.toBase58() &&
        entry.token.toBase58() === SystemProgram.programId.toBase58(),
    );

    expect(treasuryLamportsAfter - treasuryLamportsBefore).to.equal(8_000_000);
    expect(Number(publisherBalance.amount.toString())).to.equal(72_000_000);

    await base.storeProgram.methods
      .withdraw(SystemProgram.programId)
      .accounts({
        publisher: base.publisher.publicKey,
        storeState: base.storeStatePda,
      } as any)
      .signers([base.publisher])
      .rpc();

    const publisherLamportsAfter = await base.provider.connection.getBalance(
      base.publisher.publicKey,
    );
    const refreshedStoreState = (await base.storeProgram.account.storeState.fetch(
      base.storeStatePda,
    )) as any;
    const refreshedBalance = refreshedStoreState.publisherBalances.find(
      (entry: any) =>
        entry.publisher.toBase58() === base.publisher.publicKey.toBase58() &&
        entry.token.toBase58() === SystemProgram.programId.toBase58(),
    );

    expect(publisherLamportsAfter).to.be.greaterThan(publisherLamportsBefore);
    expect(Number(refreshedBalance.amount.toString())).to.equal(0);
  });
});
