import { expect } from "chai";
import { Keypair, SystemProgram, Transaction } from "@solana/web3.js";
import { getAccount, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import {
  STATUS_ACTIVE,
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("fee-exempt publish grant", () => {
  it("keeps publisher token balance unchanged when publish grant is active", async () => {
    const base = await setupPeridotFixture();
    const exemptPublisher = Keypair.generate();

    await base.provider.sendAndConfirm(
      new Transaction().add(
        SystemProgram.transfer({
          fromPubkey: base.provider.publicKey,
          toPubkey: exemptPublisher.publicKey,
          lamports: 2 * 1_000_000_000,
        }),
      ),
    );

    const ata = await getOrCreateAssociatedTokenAccount(
      base.provider.connection,
      base.authority,
      base.paymentMint,
      exemptPublisher.publicKey,
    );

    const before = (await getAccount(base.provider.connection, ata.address)).amount;

    const game = await createRegisteredGame(base, {
      publisher: exemptPublisher,
      gameId: `grant-game-${Date.now()}`,
    });

    const after = (await getAccount(base.provider.connection, ata.address)).amount;
    const registryGame = (await base.registryProgram.account.registryGame.fetch(
      game.registryGamePda,
    )) as any;

    expect(before.toString()).to.eq(after.toString());
    expect((registryGame.status as any).active).to.not.eq(undefined);
  });
});
