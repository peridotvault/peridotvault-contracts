import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { network } from "hardhat";
import { getAddress } from "viem";

// ANT-941 (FR4-ON): owner-only license revocation on PGC1.
// Deploys via PGC1Factory (implementation is M-1 locked; clones initialize).
// Covers: buy -> revoke -> balance 0 + LicenseRevoked event,
// non-owner revoke reverts, double revoke reverts NoLicenseToRevoke,
// zero-address revoke reverts ZeroAddress, re-buy after revoke works
// (indexer dedupes per contract+tokenId).
describe("PGC1 revoke (ANT-941 FR4-ON)", async function () {
  const { viem } = await network.connect();
  const publicClient = await viem.getPublicClient();

  const PRICE = 100n; // wei, native payment (paymentToken = 0x0)
  const LICENSE_ID = 1n;

  async function deployGame(gameId: string) {
    const [deployer, buyer, stranger] = await viem.getWalletClients();
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
      0n, // publishFee 0 for tests
    ]);
    await registry.write.setFactory([factory.address]);
    await factory.write.setRegistry([registry.address]);

    await factory.write.publishGame([
      {
        tokenURI1155: "ipfs://ant941-test",
        initialContractMetaHash: `0x${"11".repeat(32)}`,
        initialContractMetaURI: "ipfs://ant941-meta",
        gameId,
        paymentToken: "0x0000000000000000000000000000000000000000",
        price: PRICE,
        maxSupply: 0n, // unlimited
      },
    ]);
    const record = (await registry.read.games([gameId])) as any;
    const cloneAddr = record[0] as `0x${string}`;
    assert.ok(cloneAddr !== "0x0000000000000000000000000000000000000000");
    const pgc1 = await viem.getContractAt("PGC1", cloneAddr);

    const buyAs = (account: { address: `0x${string}` }) =>
      (pgc1.write.buy as any)([], { value: PRICE, account });
    const revokeAs = (account: { address: `0x${string}` }, holder: string) =>
      (pgc1.write.revoke as any)([holder], { account });
    return { pgc1, deployer, buyer, stranger, buyAs, revokeAs };
  }

  it("buy mints LICENSE_ID; owner revoke burns + emits LicenseRevoked", async function () {
    const { pgc1, buyer, buyAs } = await deployGame("ant941-revoke-basic");

    const buyHash = (await buyAs(buyer.account)) as `0x${string}`;
    const buyReceipt = await publicClient.waitForTransactionReceipt({
      hash: buyHash,
    });
    console.log("      buy gasUsed:", buyReceipt.gasUsed.toString());
    assert.equal(
      await pgc1.read.balanceOf([buyer.account.address, LICENSE_ID]),
      1n,
    );

    await viem.assertions.emitWithArgs(
      pgc1.write.revoke([buyer.account.address]),
      pgc1,
      "LicenseRevoked",
      [getAddress(buyer.account.address)],
    );
    assert.equal(
      await pgc1.read.balanceOf([buyer.account.address, LICENSE_ID]),
      0n,
    );
  });

  it("non-owner cannot revoke (access control: least privilege)", async function () {
    const { pgc1, buyer, stranger, buyAs, revokeAs } =
      await deployGame("ant941-revoke-acl");
    await buyAs(buyer.account);

    await viem.assertions.revertWithCustomError(
      revokeAs(stranger.account, buyer.account.address),
      pgc1,
      "OwnableUnauthorizedAccount",
    );
    assert.equal(
      await pgc1.read.balanceOf([buyer.account.address, LICENSE_ID]),
      1n,
    );
  });

  it("double revoke reverts NoLicenseToRevoke; zero address reverts", async function () {
    const { pgc1, buyer, buyAs } = await deployGame("ant941-revoke-dedupe");
    await buyAs(buyer.account);

    await pgc1.write.revoke([buyer.account.address]);
    await viem.assertions.revertWithCustomError(
      pgc1.write.revoke([buyer.account.address]),
      pgc1,
      "NoLicenseToRevoke",
    );
    await viem.assertions.revertWithCustomError(
      pgc1.write.revoke(["0x0000000000000000000000000000000000000000"]),
      pgc1,
      "ZeroAddress",
    );
  });

  it("holder can re-buy after revoke (re-mint path for indexer dedupe)", async function () {
    const { pgc1, buyer, buyAs } = await deployGame("ant941-revoke-rebuy");
    await buyAs(buyer.account);

    const revokeHash = await pgc1.write.revoke([buyer.account.address]);
    const revokeReceipt = await publicClient.waitForTransactionReceipt({
      hash: revokeHash,
    });
    console.log("      revoke gasUsed:", revokeReceipt.gasUsed.toString());

    await buyAs(buyer.account);
    assert.equal(
      await pgc1.read.balanceOf([buyer.account.address, LICENSE_ID]),
      1n,
    );
  });
});
