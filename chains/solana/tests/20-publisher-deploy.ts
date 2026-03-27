import { expect } from "chai";
import { DEFAULT_PLATFORM_FEE_BPS, setupPeridotFixture } from "./helpers/peridot";

describe("publisher deploy flow", () => {
  it("deploys and initializes registry, game-store, and pgc1 state", async () => {
    const base = await setupPeridotFixture();

    const registryState = (await base.registryProgram.account.registryState.fetch(
      base.registryStatePda,
    )) as any;
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const pgcGlobal = (await base.pgcProgram.account.globalState.fetch(base.pgcGlobalStatePda)) as any;

    expect(registryState.governance.toBase58()).to.equal(base.governance.publicKey.toBase58());
    expect(storeState.registry.toBase58()).to.equal(base.registryStatePda.toBase58());
    expect(storeState.platformFeeBps).to.equal(DEFAULT_PLATFORM_FEE_BPS);
    expect(pgcGlobal.registry.toBase58()).to.equal(base.registryProgram.programId.toBase58());
    expect(pgcGlobal.gameStore.toBase58()).to.equal(base.storeProgram.programId.toBase58());
  });
});
