import { expect } from "chai";

import {
  DEFAULT_PLATFORM_FEE_BPS,
  UPDATED_PLATFORM_FEE_BPS,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("admin config", () => {
  it("updates governance, treasury, and platform fee across programs", async () => {
    const base = await setupPeridotFixture();

    await base.registryProgram.methods
      .setGovernance(base.nextGovernance.publicKey)
      .accounts({
        governance: base.governance.publicKey,
        registryState: base.registryStatePda,
      })
      .rpc();
    await base.storeProgram.methods
      .setGovernance(base.nextGovernance.publicKey)
      .accounts({
        governance: base.governance.publicKey,
        storeState: base.storeStatePda,
      })
      .rpc();
    await base.factoryProgram.methods
      .setGovernance(base.nextGovernance.publicKey)
      .accounts({
        governance: base.governance.publicKey,
        factoryState: base.factoryStatePda,
      })
      .rpc();

    await base.registryProgram.methods
      .setTreasury(base.nextTreasury.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        registryState: base.registryStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();
    await base.storeProgram.methods
      .setTreasury(base.nextTreasury.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();
    await base.storeProgram.methods
      .setPlatformFee(UPDATED_PLATFORM_FEE_BPS)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();

    const registryState = (await base.registryProgram.account.registryState.fetch(
      base.registryStatePda,
    )) as any;
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const factoryState = (await base.factoryProgram.account.factoryState.fetch(
      base.factoryStatePda,
    )) as any;

    expect(registryState.governance.toBase58()).to.equal(
      base.nextGovernance.publicKey.toBase58(),
    );
    expect(storeState.governance.toBase58()).to.equal(
      base.nextGovernance.publicKey.toBase58(),
    );
    expect(factoryState.governance.toBase58()).to.equal(
      base.nextGovernance.publicKey.toBase58(),
    );
    expect(registryState.treasury.toBase58()).to.equal(base.nextTreasury.publicKey.toBase58());
    expect(storeState.treasury.toBase58()).to.equal(base.nextTreasury.publicKey.toBase58());
    expect(storeState.platformFeeBps).to.equal(UPDATED_PLATFORM_FEE_BPS);

    await base.registryProgram.methods
      .setTreasury(base.treasury.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        registryState: base.registryStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();
    await base.storeProgram.methods
      .setTreasury(base.treasury.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();
    await base.storeProgram.methods
      .setPlatformFee(DEFAULT_PLATFORM_FEE_BPS)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();

    await base.registryProgram.methods
      .setGovernance(base.governance.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        registryState: base.registryStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();
    await base.storeProgram.methods
      .setGovernance(base.governance.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();
    await base.factoryProgram.methods
      .setGovernance(base.governance.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        factoryState: base.factoryStatePda,
      })
      .signers([base.nextGovernance])
      .rpc();
  });
});
