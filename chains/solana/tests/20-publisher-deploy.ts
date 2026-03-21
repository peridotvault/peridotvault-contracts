import { expect } from "chai";

import { DEFAULT_PLATFORM_FEE_BPS, setupPeridotFixture } from "./helpers/peridot";

describe("publisher deploy flow", () => {
  it("deploys and initializes registry, game-store, and factory state", async () => {
    const base = await setupPeridotFixture();

    const registryState = (await base.registryProgram.account.registryState.fetch(
      base.registryStatePda,
    )) as any;
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const factoryState = (await base.factoryProgram.account.factoryState.fetch(
      base.factoryStatePda,
    )) as any;

    expect(registryState.governance.toBase58()).to.equal(base.governance.publicKey.toBase58());
    expect(registryState.factory.toBase58()).to.equal(base.factoryStatePda.toBase58());
    expect(storeState.registry.toBase58()).to.equal(base.registryStatePda.toBase58());
    expect(storeState.platformFeeBps).to.equal(DEFAULT_PLATFORM_FEE_BPS);
    expect(factoryState.registry.toBase58()).to.equal(base.registryStatePda.toBase58());
    expect(factoryState.gameStore.toBase58()).to.equal(base.storeStatePda.toBase58());
  });
});
