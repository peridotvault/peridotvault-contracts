import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { network } from "hardhat";

/**
 * ANT-886 (M-1): PGC1 implementation contract locks `initialize`.
 * - Direct `initialize` on the implementation reverts `AlreadyInitialized`.
 * - Clones from the factory still initialize (lock is per-proxy storage).
 * Domain lenses: Upgradeability safety (initializer discipline on clones),
 * Access control (no seizure of the logic contract).
 */
describe("PGC1 implementation lock (ANT-886 M-1)", async function () {
  const { viem } = await network.connect();

  async function deploySystem() {
    const [deployer] = await viem.getWalletClients();
    const treasury = await viem.deployContract("PeridotTreasuryRouter", [
      deployer.account.address,
      [deployer.account.address],
    ]);
    const registry = await viem.deployContract("PeridotRegistry", []);
    const pgc1Impl = await viem.deployContract("PGC1", []);
    const factory = await viem.deployContract("PGC1Factory", [
      pgc1Impl.address,
      treasury.address,
      "0x0000000000000000000000000000000000000000",
      0n,
    ]);
    await registry.write.setFactory([factory.address]);
    await factory.write.setRegistry([registry.address]);
    return { deployer, treasury, registry, pgc1Impl, factory };
  }

  function initFor(gameId: string) {
    return {
      tokenURI1155: "ipfs://impl-lock-test",
      initialContractMetaHash:
        "0x0000000000000000000000000000000000000000000000000000000000000001",
      initialContractMetaURI: "ipfs://impl-lock-meta",
      gameId,
      paymentToken: "0x0000000000000000000000000000000000000000",
      price: 100n,
      maxSupply: 10n,
    };
  }

  it("implementation initialize reverts AlreadyInitialized", async function () {
    const { deployer, treasury, pgc1Impl } = await deploySystem();
    await assert.rejects(
      () =>
        pgc1Impl.write.initialize([
          "ipfs://x",
          "0x0000000000000000000000000000000000000000000000000000000000000001",
          "ipfs://x",
          "impl-seize",
          "0x0000000000000000000000000000000000000000",
          100n,
          10n,
          treasury.address,
          deployer.account.address,
          500,
          deployer.account.address,
        ]),
      /AlreadyInitialized/,
    );
  });

  it("clones still initialize via publishGame", async function () {
    const { factory, registry } = await deploySystem();
    await factory.write.publishGame([initFor("impl-lock-clone")]);
    const record = (await registry.read.games(["impl-lock-clone"])) as any;
    const cloneAddr = record[0] as `0x${string}`;
    assert.ok(cloneAddr !== "0x0000000000000000000000000000000000000000");
    const pgc1 = await viem.getContractAt("PGC1", cloneAddr);
    assert.equal(await pgc1.read.price(), 100n);
  });
});
