import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { network } from "hardhat";

/**
 * ANT-478 M0 impl — EVM negative + version tests (AC-H2-3, AC-H3-5, AC-M2-1/2, AC-M5-1/2).
 * Runs on the ephemeral in-process Hardhat chain (no mainnet, no value-bearing deploys).
 */
describe("M0 hardening (EVM)", async function () {
  const { viem } = await network.connect();

  async function deploySystem() {
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
      0n,
    ]);
    await registry.write.setFactory([factory.address]);
    await factory.write.setRegistry([registry.address]);
    return { deployer, buyer, stranger, treasury, registry, pgc1Impl, factory };
  }

  async function publishEthGame(
    factory: any,
    gameId: string,
    price: bigint,
  ): Promise<any> {
    const init = {
      tokenURI1155: "ipfs://m0-test",
      initialContractMetaHash:
        "0x0000000000000000000000000000000000000000000000000000000000000001",
      initialContractMetaURI: "ipfs://m0-meta",
      gameId,
      paymentToken: "0x0000000000000000000000000000000000000000",
      price,
      maxSupply: 100n,
    };
    const hash = await factory.write.publishGame([init]);
    void hash;
    return init;
  }

  it("AC-H2-3: no public mint entry point; unpaid buy reverts, paid buy mints", async function () {
    const { factory, registry, buyer } = await deploySystem();
    await publishEthGame(factory, "m0-h2", 100n);

    const record = (await registry.read.games(["m0-h2"])) as any;
    const cloneAddr = record[0] as `0x${string}`;
    assert.ok(cloneAddr !== "0x0000000000000000000000000000000000000000");
    const pgc1 = await viem.getContractAt("PGC1", cloneAddr);

    // H2 structural gate: the clone exposes `buy`/`burn` only — no `mint(` function.
    const fnNames = (pgc1.abi as any[])
      .filter((e) => e.type === "function")
      .map((e) => e.name);
    assert.ok(fnNames.includes("buy"), "buy entry point exists");
    assert.ok(
      !fnNames.some((n) => n.toLowerCase().startsWith("mint")),
      `no public mint function (found: ${fnNames.join(",")})`,
    );

    // Caller without payment reverts; the authorized purchase path mints.
    await assert.rejects(
      () => (pgc1.write.buy as any)([], { value: 0n, account: buyer.account }),
      /InvalidPayment|FreePathNotAllowed/,
    );
    await (pgc1.write.buy as any)([], { value: 100n, account: buyer.account });
    assert.equal(await pgc1.read.balanceOf([buyer.account.address, 1n]), 1n);
  });

  it("AC-H3-5 + AC-M5-1: paid/zero-payment and free-flag lifecycle with version checks", async function () {
    const { factory, registry, buyer, deployer } = await deploySystem();
    const price = 1_000n;
    await publishEthGame(factory, "m0-h3", price);

    const record = (await registry.read.games(["m0-h3"])) as any;
    const cloneAddr = record[0] as `0x${string}`;
    assert.ok(cloneAddr !== "0x0000000000000000000000000000000000000000");

    const pgc1 = await viem.getContractAt("PGC1", cloneAddr);

    // M5: version constants present.
    assert.equal(await pgc1.read.contractVersion([]), 1);
    assert.equal(await pgc1.read.attestationVersion([]), 1);
    assert.equal(await pgc1.read.pgcVersion([]), 1n);
    // H3: paid by default.
    assert.equal(await pgc1.read.isFree([]), false);

    // Paid game + zero payment reverts (H3 two-sided invariant).
    await assert.rejects(
      () => (pgc1.write.buy as any)([], { value: 0n, account: buyer.account }),
      /InvalidPayment|FreePathNotAllowed/,
    );

    // Paid game + exact payment mints.
    await (pgc1.write.buy as any)([], { value: price, account: buyer.account });
    assert.equal(
      await pgc1.read.balanceOf([buyer.account.address, 1n]),
      1n,
    );

    // setPrice(0) on a paid game reverts (no ambiguous middle).
    await assert.rejects(() => pgc1.write.setPrice([0n]), /FreePathNotAllowed/);

    // Owner marks free: price resets to 0 atomically.
    await pgc1.write.setIsFree([true]);
    assert.equal(await pgc1.read.isFree([]), true);
    assert.equal(await pgc1.read.price([]), 0n);

    // Free game + zero payment mints for a fresh buyer (AC-H3-5 free side).
    const wallets = await viem.getWalletClients();
    const strangerWallet = wallets[2] ?? wallets[1];
    await (pgc1.write.buy as any)([], { value: 0n, account: strangerWallet.account });
    assert.equal(
      await pgc1.read.balanceOf([strangerWallet.account.address, 1n]),
      1n,
    );

    // Free game + payment attempt reverts for a buyer who does NOT own yet
    // (no ambiguous middle) — the deployer/owner never bought this game.
    await assert.rejects(
      () => (pgc1.write.buy as any)([], { value: 1n, account: deployer.account }),
      /FreeGamePaymentNotAllowed/,
    );
  });

  it("AC-M2-1: sweepToken succeeds with USDT-style no-return token", async function () {
    const { treasury, deployer } = await deploySystem();
    const weird = await viem.deployContract("NoReturnToken", []);
    await weird.write.mint([treasury.address, 500n]);
    assert.equal(await weird.read.balanceOf([treasury.address]), 500n);

    const [rec] = (await treasury.read.recipient([])) as [string];
    void rec;
    await treasury.write.sweepToken([weird.address]);
    assert.equal(await weird.read.balanceOf([treasury.address]), 0n);
    void deployer;
  });

  it("AC-M2-2: native/token sweeps revert on zero amount and zero address", async function () {
    const { treasury } = await deploySystem();

    // Empty router: sweepETH reverts ZeroAmount.
    await assert.rejects(() => treasury.write.sweepETH([]), /ZeroAmount/);

    // Zero token address reverts ZeroAddress.
    await assert.rejects(
      () =>
        treasury.write.sweepToken([
          "0x0000000000000000000000000000000000000000",
        ]),
      /ZeroAddress/,
    );

    // Zero balance of a real token reverts ZeroAmount.
    const mock = await viem.deployContract("MockERC20", []);
    await assert.rejects(() => treasury.write.sweepToken([mock.address]), /ZeroAmount/);
  });

  it("AC-M5-1/2: registry record carries RECORD_VERSION=1", async function () {
    const { factory, registry } = await deploySystem();
    await publishEthGame(factory, "m0-m5", 10n);
    assert.equal(await registry.read.REGISTRY_VERSION([]), 1);
    assert.equal(await registry.read.RECORD_VERSION([]), 1);
    const record = (await registry.read.games(["m0-m5"])) as any;
    assert.equal(record[4], 1); // version field
    const fetched = (await registry.read.getGame(["m0-m5"])) as any;
    assert.equal(fetched.version, 1);
  });
});
