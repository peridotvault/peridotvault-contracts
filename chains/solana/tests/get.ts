import { expect } from "chai";
import {
  createRegisteredGame,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("PeridotVault - read checks", () => {
  it("reads core state and can list registry/store entries", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

    const pglConfig = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;
    const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
      base.registryConfigPda,
    )) as any;
    const storeConfig = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;

    expect(pglConfig.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
    expect(registryConfig.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
    expect(storeConfig.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());

    const games = await base.registryProgram.account.registryGame.all();
    expect(games.length).to.be.greaterThan(0);

    const target = games.find((g: any) => g.publicKey.toBase58() === game.registryGamePda.toBase58());
    expect(target).to.not.eq(undefined);
  });
});
