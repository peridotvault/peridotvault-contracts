import { expect } from "chai";
import {
  DEFAULT_PLATFORM_FEE_BPS,
  UPDATED_PLATFORM_FEE_BPS,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("admin config", () => {
  it("updates governance, treasury, and platform fee across programs", async () => {
    const base = await setupPeridotFixture();

    // 1. Update Governance to nextGovernance
    await base.registryProgram.methods
      .setGovernance(base.nextGovernance.publicKey)
      .accounts({
        governance: base.governance.publicKey,
        registryState: base.registryStatePda,
      } as any)
      .signers([base.governance])
      .rpc();

    await base.storeProgram.methods
      .setGovernance(base.nextGovernance.publicKey)
      .accounts({
        governance: base.governance.publicKey,
        storeState: base.storeStatePda,
      } as any)
      .signers([base.governance])
      .rpc();

    await base.pgc1Program.methods
      .setGovernance(base.nextGovernance.publicKey)
      .accounts({
        governance: base.governance.publicKey,
        globalState: base.pgcGlobalStatePda,
      } as any)
      .signers([base.governance])
      .rpc();

    // 2. Update Treasury using nextGovernance as signer
    await base.registryProgram.methods
      .setTreasury(base.nextTreasury.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        registryState: base.registryStatePda,
      } as any)
      .signers([base.nextGovernance])
      .rpc();

    await base.storeProgram.methods
      .setTreasury(base.nextTreasury.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      } as any)
      .signers([base.nextGovernance])
      .rpc();

    await base.storeProgram.methods
      .setPlatformFee(UPDATED_PLATFORM_FEE_BPS)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      } as any)
      .signers([base.nextGovernance])
      .rpc();

    // 3. Verify changes
    const registryState = (await base.registryProgram.account.registryState.fetch(base.registryStatePda)) as any;
    const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
    const pgcGlobalState = (await base.pgc1Program.account.globalState.fetch(base.pgcGlobalStatePda)) as any;

    expect(registryState.governance.toBase58()).to.equal(base.nextGovernance.publicKey.toBase58());
    expect(storeState.governance.toBase58()).to.equal(base.nextGovernance.publicKey.toBase58());
    expect(pgcGlobalState.governance.toBase58()).to.equal(base.nextGovernance.publicKey.toBase58());
    expect(registryState.treasury.toBase58()).to.equal(base.nextTreasury.publicKey.toBase58());
    expect(storeState.treasury.toBase58()).to.equal(base.nextTreasury.publicKey.toBase58());
    expect(storeState.platformFeeBps).to.equal(UPDATED_PLATFORM_FEE_BPS);

    // 4. Cleanup/Restore for other tests
    await base.registryProgram.methods
      .setGovernance(base.governance.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        registryState: base.registryStatePda,
      } as any)
      .signers([base.nextGovernance])
      .rpc();

    await base.storeProgram.methods
      .setGovernance(base.governance.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        storeState: base.storeStatePda,
      } as any)
      .signers([base.nextGovernance])
      .rpc();

    await base.pgc1Program.methods
      .setGovernance(base.governance.publicKey)
      .accounts({
        governance: base.nextGovernance.publicKey,
        globalState: base.pgcGlobalStatePda,
      } as any)
      .signers([base.nextGovernance])
      .rpc();

    // Restore platform fee
    await base.storeProgram.methods
      .setPlatformFee(DEFAULT_PLATFORM_FEE_BPS)
      .accounts({
        governance: base.governance.publicKey,
        storeState: base.storeStatePda,
      } as any)
      .signers([base.governance])
      .rpc();
  });
});
