import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import {
  setupPeridotFixture,
  ensureGameCreated,
  deriveGameFixture,
  MINTER_AUTH_SEED,
  LICENSE_SEED,
  TEST_GAME_ID
} from "./helpers/peridot";

describe("remediation regressions", () => {
  it("removes old governance admin moderation access after governance transfer", async () => {
    const base = await setupPeridotFixture();
    await ensureGameCreated(base);

    await base.registryProgram.methods
      .setGovernance(base.nextGovernance.publicKey)
      .accounts({
        governance: base.governance.publicKey,
        registryState: base.registryStatePda,
      } as any)
      .signers([base.governance])
      .rpc();

    let failed = false;
    try {
      await base.registryProgram.methods
        .setStatus(TEST_GAME_ID, 1)
        .accounts({
          admin: base.governance.publicKey,
          registryState: base.registryStatePda,
          gameRegistration: deriveGameFixture(base, TEST_GAME_ID).gameRegistrationPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([base.governance])
        .rpc();
    } catch (error) {
      failed = true;
    }

    expect(failed).to.equal(true);

    // Restore
    await base.registryProgram.methods
      .setGovernance(base.governance.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        registryState: base.registryStatePda,
      } as any)
      .signers([base.nextGovernance])
      .rpc();
  });

  it("revokes old publisher mint power when publisher changes", async () => {
    const base = await setupPeridotFixture();
    const game = await ensureGameCreated(base);
    const newPublisher = Keypair.generate();
    const freshUser = Keypair.generate();

    await base.provider.sendAndConfirm(
      new Transaction().add(
        SystemProgram.transfer({
          fromPubkey: base.provider.publicKey,
          toPubkey: newPublisher.publicKey,
          lamports: 2 * anchor.web3.LAMPORTS_PER_SOL,
        }),
      ),
    );
    await base.provider.sendAndConfirm(
      new Transaction().add(
        SystemProgram.transfer({
          fromPubkey: base.provider.publicKey,
          toPubkey: freshUser.publicKey,
          lamports: 2 * anchor.web3.LAMPORTS_PER_SOL,
        }),
      ),
    );

    const oldPublisherMinterAuth = PublicKey.findProgramAddressSync(
      [Buffer.from("minter_auth"), game.gameStatePda.toBuffer(), base.publisher.publicKey.toBuffer()],
      base.pgc1Program.programId,
    )[0];
    const newPublisherMinterAuth = PublicKey.findProgramAddressSync(
      [Buffer.from("minter_auth"), game.gameStatePda.toBuffer(), newPublisher.publicKey.toBuffer()],
      base.pgc1Program.programId,
    )[0];
    const licensePda = PublicKey.findProgramAddressSync(
      [LICENSE_SEED, game.gameStatePda.toBuffer(), freshUser.publicKey.toBuffer()],
      base.pgc1Program.programId,
    )[0];
    
    const gameState = await base.pgc1Program.account.gameState.fetch(game.gameStatePda);
    const userGameTokenAccount = getAssociatedTokenAddressSync(
      gameState.mint,
      freshUser.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    await base.pgc1Program.methods
      .setPublisher()
      .accounts({
        publisher: base.publisher.publicKey,
        gameState: game.gameStatePda,
        newPublisher: newPublisher.publicKey,
        oldPublisherMinterAuth,
        newPublisherMinterAuth,
      } as any)
      .signers([base.publisher])
      .rpc();

    let failed = false;
    try {
      await base.pgc1Program.methods
        .mintLicense(new anchor.BN(0))
        .accounts({
          payer: base.publisher.publicKey,
          signer: base.publisher.publicKey,
          gameState: game.gameStatePda,
          user: freshUser.publicKey,
          gameAuthority: game.gameAuthorityPda,
          minterAuth: oldPublisherMinterAuth,
          mint: gameState.mint,
          licenseAccount: licensePda,
          userTokenAccount: userGameTokenAccount,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([base.publisher])
        .rpc();
    } catch (error) {
      failed = true;
    }

    expect(failed).to.equal(true);
  });
});
