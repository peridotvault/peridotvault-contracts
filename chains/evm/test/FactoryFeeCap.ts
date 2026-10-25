import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { network } from "hardhat";

/**
 * ANT-505 (parent ANT-482): factory fee cap must share the PGC1 2000 cap.
 * - setPlatformFeeBps(2001) and (10000) revert with InvalidPlatformFeeBps
 * - setPlatformFeeBps(2000) succeeds
 * - publishGame after setPlatformFeeBps(2000) initializes OK
 */
describe("PGC1Factory fee cap (ANT-505)", async function () {
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
      tokenURI1155: "ipfs://fee-cap-test",
      initialContractMetaHash:
        "0x0000000000000000000000000000000000000000000000000000000000000001",
      initialContractMetaURI: "ipfs://fee-cap-meta",
      gameId,
      paymentToken: "0x0000000000000000000000000000000000000000",
      price: 100n,
      maxSupply: 10n,
    };
  }

  it("reverts setPlatformFeeBps(2001) and (10000)", async function () {
    const { factory } = await deploySystem();
    await assert.rejects(
      () => factory.write.setPlatformFeeBps([2001]),
      /InvalidPlatformFeeBps/,
    );
    await assert.rejects(
      () => factory.write.setPlatformFeeBps([10000]),
      /InvalidPlatformFeeBps/,
    );
  });

  it("accepts 2000 and publishGame initializes OK", async function () {
    const { factory, registry } = await deploySystem();
    await factory.write.setPlatformFeeBps([2000]);
    assert.equal(await factory.read.platformFeeBps(), 2000);

    await factory.write.publishGame([initFor("fee-cap-2000")]);
    const record = (await registry.read.games(["fee-cap-2000"])) as any;
    const cloneAddr = record[0] as `0x${string}`;
    assert.ok(cloneAddr !== "0x0000000000000000000000000000000000000000");

    const pgc1 = await viem.getContractAt("PGC1", cloneAddr);
    assert.equal(await pgc1.read.platformFeeBps(), 2000);
  });
});
