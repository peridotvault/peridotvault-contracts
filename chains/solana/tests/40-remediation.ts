import { expect } from "chai";
import {
  STATUS_ACTIVE,
  STATUS_BANNED,
  STATUS_SUSPENDED,
  createRegisteredGame,
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
});
