import { expect } from "chai";
import {
  STATUS_ACTIVE,
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("publisher create game", () => {
  it("creates game in pgl1 and registers it in registry", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

    const pglGame = (await base.pglProgram.account.game.fetch(game.gamePda)) as any;
    const registryGame = (await base.registryProgram.account.registryGame.fetch(
      game.registryGamePda,
    )) as any;

    expect(pglGame.gameId).to.eq(game.gameId);
    expect(pglGame.metadataUri).to.eq(game.metadataUri);
    expect(pglGame.creator.toBase58()).to.eq(game.publisher.publicKey.toBase58());
    expect(pglGame.publisher.toBase58()).to.eq(game.publisher.publicKey.toBase58());

    expect(registryGame.game.toBase58()).to.eq(game.gamePda.toBase58());
    expect(registryGame.gameId).to.eq(game.gameId);
    expect((registryGame.status as any).active).to.not.eq(undefined);
  });
});
