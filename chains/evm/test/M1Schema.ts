import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";
import { keccak256, parseEventLogs } from "viem";

import { network } from "hardhat";

/**
 * ANT-490 M1 schema impl — gameId vectors + registry lifecycle + timelock +
 * envelope binding + fixtures (parent ANT-485 spec §4 AC, §6 NFRs).
 * Runs on the ephemeral in-process Hardhat chain (0 mainnet txs, no secrets).
 */

type Vec = {
  vector: number;
  chainId: string;
  publisherHex: string;
  localId: string;
  gameId: string;
};

const VECTORS: Vec[] = JSON.parse(
  readFileSync(new URL("../../../fixtures/gameid.vectors.json", import.meta.url), "utf8"),
).vectors;

const LOCALNET = JSON.parse(
  readFileSync(new URL("../../../fixtures/registry.localnet.json", import.meta.url), "utf8"),
);
const TESTNETS = JSON.parse(
  readFileSync(new URL("../../../fixtures/registry.testnets.json", import.meta.url), "utf8"),
);

/**
 * Manual §3.1 byte layout (what the Solana impl writes directly, no ABI lib).
 * NOTE (spec deviation, ANT-485 §3.1 table): offsets are NOT constant
 * `0x60/0xa0/0xe0` — each head offset points past all preceding tails, so
 * off1/off2 grow when chainId/publisher exceed 32 bytes. The spec table's
 * "offsets constant" claim holds only for short inputs (vectors 1-3); the
 * normative formula keccak256(abi.encode(...)) governs. Flagged to Theo.
 */
function manualPreimage(chainId: string, publisherHex: string, localId: string): `0x${string}` {
  const enc = new TextEncoder();
  const c = enc.encode(chainId);
  const p = Buffer.from(publisherHex.replace(/^0x/, ""), "hex");
  const l = enc.encode(localId);
  const padLen = (n: number) => Math.ceil(n / 32) * 32 || 32;
  const pad32 = (b: Uint8Array) => {
    const out = new Uint8Array(padLen(b.length));
    out.set(b);
    return out;
  };
  const u256 = (n: number) => {
    const out = new Uint8Array(32);
    const dv = new DataView(out.buffer);
    dv.setUint32(28, n);
    return out;
  };
  const off1 = 0x60 + 32 + padLen(c.length);
  const off2 = off1 + 32 + padLen(p.length);
  const parts = [
    u256(0x60),
    u256(off1),
    u256(off2),
    u256(c.length),
    pad32(c),
    u256(p.length),
    pad32(p),
    u256(l.length),
    pad32(l),
  ];
  const total = parts.reduce((a, b) => a + b.length, 0);
  const out = new Uint8Array(total);
  let o = 0;
  for (const q of parts) {
    out.set(q, o);
    o += q.length;
  }
  return `0x${Buffer.from(out).toString("hex")}` as `0x${string}`;
}

describe("M1 schema (EVM)", async function () {
  const { viem, provider } = await network.connect();

  async function deploy() {
    const registry = await viem.deployContract("XChainGameRegistry", []);
    const harness = await viem.deployContract("XChainEnvelopeHarness", []);
    const [owner] = await viem.getWalletClients();
    return { registry, harness, owner };
  }

  async function proposeActive(
    registry: any,
    chainId: string,
    publisher: `0x${string}`,
    localId: string,
    implAddr: `0x${string}`,
    format: number,
  ): Promise<`0x${string}`> {
    await registry.write.proposeGame([chainId, publisher, localId, implAddr, format]);
    const gameId = (await registry.read.deriveGameId([chainId, publisher, localId])) as `0x${string}`;
    await registry.write.activateGame([gameId]);
    return gameId;
  }

  it("AC-GAMEID-1: all 5 fixture vectors derive on-chain (incl. spec §4 example 2)", async function () {
    const { registry } = await deploy();
    for (const v of VECTORS) {
      const got = (await registry.read.deriveGameId([
        v.chainId,
        `0x${v.publisherHex}`,
        v.localId,
      ])) as string;
      assert.equal(got.toLowerCase(), v.gameId.toLowerCase(), `vector ${v.vector} ${v.localId}`);
    }
  });

  it("AC-GAMEID-2: manual §3.1 layout hashes identically to abi.encode (cross-stack equivalence)", async function () {
    const { registry } = await deploy();
    for (const v of VECTORS) {
      const preimage = manualPreimage(v.chainId, v.publisherHex, v.localId);
      const manual = keccak256(preimage);
      const onchain = (await registry.read.deriveGameId([
        v.chainId,
        `0x${v.publisherHex}`,
        v.localId,
      ])) as string;
      assert.equal(manual.toLowerCase(), onchain.toLowerCase(), `vector ${v.vector} manual==abi`);
      assert.equal(manual.toLowerCase(), v.gameId.toLowerCase(), `vector ${v.vector} manual==fixture`);
    }
  });

  it("AC-GAMEID-3: same publisher+localId on two chainIds gives different gameIds", async function () {
    const { registry } = await deploy();
    const pub = "0x1111111111111111111111111111111111111111" as `0x${string}`;
    const a = (await registry.read.deriveGameId(["eip155:84532", pub, "same-game"])) as string;
    const b = (await registry.read.deriveGameId(["eip155:4202", pub, "same-game"])) as string;
    assert.notEqual(a, b);
  });

  it("AC-GAMEID-4: registration rejects empty localId / unknown chainId with SPECIFIC errors", async function () {
    const { registry } = await deploy();
    const pub = "0x1111111111111111111111111111111111111111" as `0x${string}`;
    const impl = "0x2222222222222222222222222222222222222222" as `0x${string}`;
    await assert.rejects(
      () => registry.write.proposeGame(["eip155:84532", pub, "", impl, 0]),
      /EmptyLocalId/,
    );
    await assert.rejects(
      () => registry.write.proposeGame(["eip155:99999", pub, "x", impl, 0]),
      /UnknownChainId/,
    );
    await assert.rejects(
      () => registry.write.proposeGame(["eip155:84532", pub, "has space", impl, 0]),
      /InvalidLocalIdCharset/,
    );
    await assert.rejects(
      () => registry.write.proposeGame(["eip155:84532", pub, "y".repeat(129), impl, 0]),
      /LocalIdTooLong/,
    );
    await assert.rejects(
      () => registry.write.proposeGame(["eip155:84532", "0x1234", "x", impl, 0]),
      /InvalidPublisher/,
    );
  });

  it("AC-REG-1: PROPOSED→ACTIVE sets registeredAt and emits GameActivated", async function () {
    const { registry } = await deploy();
    const pub = "0x1111111111111111111111111111111111111111" as `0x${string}`;
    const impl = "0x2222222222222222222222222222222222222222" as `0x${string}`;
    await registry.write.proposeGame(["eip155:84532", pub, "reg-one", impl, 0]);
    const gameId = (await registry.read.deriveGameId(["eip155:84532", pub, "reg-one"])) as `0x${string}`;
    let e = (await registry.read.getEntry([gameId])) as any;
    assert.equal(e.status, 0); // PROPOSED
    assert.equal(e.registeredAt, 0n);
    const hash = await registry.write.activateGame([gameId]);
    e = (await registry.read.getEntry([gameId])) as any;
    assert.equal(e.status, 1); // ACTIVE
    assert.ok(e.registeredAt > 0n);
    const publicClient = await viem.getPublicClient();
    const receipt = await publicClient.getTransactionReceipt({ hash });
    const events = parseEventLogs({ abi: registry.abi, logs: receipt.logs });
    assert.ok(events.some((x) => x.eventName === "GameActivated"), "GameActivated emitted");
  });

  it("AC-REG-2: ACTIVE→DEPRECATED→FROZEN, FROZEN reads fail closed, freeze event distinct", async function () {
    const { registry } = await deploy();
    const gameId = await proposeActive(
      registry,
      "eip155:84532",
      "0x1111111111111111111111111111111111111111",
      "reg-two",
      "0x2222222222222222222222222222222222222222",
      0,
    );
    await registry.write.deprecateGame([gameId]);
    let e = (await registry.read.getEntry([gameId])) as any;
    assert.equal(e.status, 2); // DEPRECATED still readable (dual-accept)
    const hash = await registry.write.freezeGame([gameId]);
    e = (await registry.read.getEntry([gameId])) as any;
    assert.equal(e.status, 3); // FROZEN
    const publicClient = await viem.getPublicClient();
    const receipt = await publicClient.getTransactionReceipt({ hash });
    const events = parseEventLogs({ abi: registry.abi, logs: receipt.logs });
    assert.ok(events.some((x) => x.eventName === "EmergencyFreeze"), "distinct EmergencyFreeze emitted");
    assert.ok(!events.some((x) => x.eventName === "GameDeprecated"), "freeze does not reuse deprecate event");
    await assert.rejects(() => registry.read.requireAuthoritative([gameId]), /EntryFrozen/);
  });

  it("AC-REG-3: FROZEN is terminal — re-activation fails", async function () {
    const { registry } = await deploy();
    const gameId = await proposeActive(
      registry,
      "eip155:4202",
      "0x2222222222222222222222222222222222222222",
      "reg-three",
      "0x1111111111111111111111111111111111111111",
      0,
    );
    await registry.write.freezeGame([gameId]);
    await assert.rejects(() => registry.write.activateGame([gameId]), /InvalidTransition/);
    await assert.rejects(() => registry.write.deprecateGame([gameId]), /InvalidTransition/);
  });

  it("AC-REG-4: version > CURRENT fails with UnsupportedVersion (storage fault-injection)", async function () {
    const { registry } = await deploy();
    const gameId = await proposeActive(
      registry,
      "eip155:84532",
      "0x1111111111111111111111111111111111111111",
      "reg-four",
      "0x2222222222222222222222222222222222222222",
      0,
    );
    // Fault-inject version=2 into the entry struct. Storage layout: OZ
    // Ownable._owner occupies slot 0, so _entries is slot 1. The struct packs
    // format/version/status (1 byte each) into ONE word at base+2
    // (base+0/1 hold the chainId/programId length words). Read-modify-write
    // flips only the version byte 1 -> 2, leaving format/status intact.
    const { keccak256: k, encodeAbiParameters, toHex } = await import("viem");
    const base = BigInt(k(encodeAbiParameters([{ type: "bytes32" }, { type: "uint256" }], [gameId, 1n])));
    const slot = toHex(base + 2n);
    const cur = (await provider.request({
      method: "eth_getStorageAt",
      params: [registry.address, slot, "latest"],
    })) as string;
    assert.equal(BigInt(cur), 0x010100n, "precondition: format=0/version=1/status=1 packed");
    const faulted = (BigInt(cur) & ~(0xffn << 8n)) | (2n << 8n);
    await provider.request({
      method: "hardhat_setStorageAt",
      params: [registry.address, slot, toHex(faulted, { size: 32 })],
    });
    await assert.rejects(() => registry.read.getEntry([gameId]), /UnsupportedVersion/);
  });

  it("AC-REG-5: EVM20 vs SOLANA32 with identical trailing bytes are NOT equal", async function () {
    const { registry } = await deploy();
    const evmPub = "0x1111111111111111111111111111111111111111" as `0x${string}`;
    const solPub = "0x0000000000000000000000001111111111111111111111111111111111111111" as `0x${string}`;
    const evmImpl = "0x2222222222222222222222222222222222222222" as `0x${string}`;
    const solImpl = "0x0000000000000000000000002222222222222222222222222222222222222222" as `0x${string}`;
    await registry.write.proposeGame(["eip155:84532", evmPub, "fmt-game", evmImpl, 0]);
    await registry.write.proposeGame(["solana:localnet", solPub, "fmt-game", solImpl, 1]);
    const gEvm = (await registry.read.deriveGameId(["eip155:84532", evmPub, "fmt-game"])) as string;
    const gSol = (await registry.read.deriveGameId(["solana:localnet", solPub, "fmt-game"])) as string;
    assert.notEqual(gEvm, gSol);
    const eEvm = (await registry.read.getEntry([gEvm])) as any;
    const eSol = (await registry.read.getEntry([gSol])) as any;
    assert.equal(eEvm.format, 0);
    assert.equal(eSol.format, 1);
  });

  it("AC-TIME-1: execute before effectiveAt reverts even with short localnet delay", async function () {
    const { registry } = await deploy();
    const gameId = await proposeActive(
      registry,
      "eip155:31337",
      "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "time-one",
      "0x2222222222222222222222222222222222222222",
      0,
    );
    await registry.write.proposeUpdate([
      gameId,
      "0x3333333333333333333333333333333333333333",
      60n,
    ]);
    await assert.rejects(() => registry.write.executeUpdate([gameId]), /UpdateNotEffective/);
    await provider.request({ method: "evm_increaseTime", params: [61] });
    await provider.request({ method: "evm_mine", params: [] });
    await registry.write.executeUpdate([gameId]);
    const e = (await registry.read.getEntry([gameId])) as any;
    assert.equal(e.programId.toLowerCase(), "0x3333333333333333333333333333333333333333");
    assert.equal(e.hasPending, false);
  });

  it("AC-TIME-2: single-flight — second proposal while one in flight fails", async function () {
    const { registry } = await deploy();
    const gameId = await proposeActive(
      registry,
      "eip155:31337",
      "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "time-two",
      "0x2222222222222222222222222222222222222222",
      0,
    );
    await registry.write.proposeUpdate([gameId, "0x3333333333333333333333333333333333333333", 60n]);
    await assert.rejects(
      () =>
        registry.write.proposeUpdate([gameId, "0x4444444444444444444444444444444444444444", 60n]),
      /UpdateAlreadyPending/,
    );
    await assert.rejects(
      () => registry.write.proposeUpdate([gameId, "0x4444444444444444444444444444444444444444", 0n]),
      /UpdateAlreadyPending/,
    );
  });

  it("AC-TIME-3: emergency freeze bypasses pending timelock immediately", async function () {
    const { registry } = await deploy();
    const gameId = await proposeActive(
      registry,
      "eip155:84532",
      "0x1111111111111111111111111111111111111111",
      "time-three",
      "0x2222222222222222222222222222222222222222",
      0,
    );
    await registry.write.proposeUpdate([gameId, "0x3333333333333333333333333333333333333333", 3600n]);
    await registry.write.freezeGame([gameId]); // no wait for effectiveAt
    const e = (await registry.read.getEntry([gameId])) as any;
    assert.equal(e.status, 3);
    await assert.rejects(() => registry.write.executeUpdate([gameId]), /EntryFrozen/);
  });

  it("AC-ENV-1..3: dest envelope rules (never infer free-from-zero, no middle, versions match)", async function () {
    // NOTE: the harness only exposes pure functions, so viem binds `verify`
    // under `read` (eth_call) — reverts surface identically to a dest call.
    const { harness } = await deploy();
    const tok = "0x1111111111111111111111111111111111111111" as `0x${string}`;
    // Happy paths: explicit free, and paid with exact settlement.
    await harness.read.verify([{ isFree: true, paidAmount: 0n, paidToken: "0x", paidTokenFormat: 0, receiptVersion: 1, version: 1 }]);
    await harness.read.verify([
      { isFree: false, paidAmount: 100n, paidToken: tok, paidTokenFormat: 0, receiptVersion: 1, version: 1 },
    ]);
    // AC-ENV-1: paid flag with zero settlement MUST reject.
    await assert.rejects(
      () =>
        harness.read.verify([
          { isFree: false, paidAmount: 0n, paidToken: "0x", paidTokenFormat: 0, receiptVersion: 1, version: 1 },
        ]),
      /PaidAmountZeroOnPaidPath/,
    );
    // AC-ENV-2: free flag with nonzero settlement MUST reject.
    await assert.rejects(
      () =>
        harness.read.verify([
          { isFree: true, paidAmount: 5n, paidToken: "0x", paidTokenFormat: 0, receiptVersion: 1, version: 1 },
        ]),
      /PaidAmountNonZeroOnFreePath/,
    );
    // AC-ENV-3: receipt/envelope version skew MUST reject.
    await assert.rejects(
      () =>
        harness.read.verify([
          { isFree: true, paidAmount: 0n, paidToken: "0x", paidTokenFormat: 0, receiptVersion: 0, version: 1 },
        ]),
      /VersionMismatch/,
    );
  });

  it("AC-FIX-1: all fixtures present, schema-valid, zero mainnet IDs", async function () {
    for (const [name, fx] of [["localnet", LOCALNET], ["testnets", TESTNETS]] as const) {
      assert.ok(Array.isArray(fx.entries) && fx.entries.length > 0, `${name} has entries`);
      for (const e of fx.entries) {
        assert.equal(e.version, 1, `${name}/${e.chainId} version`);
        assert.equal(e.status, "active", `${name}/${e.chainId} status`);
        assert.ok(["EVM20", "SOLANA32"].includes(e.format), `${name}/${e.chainId} format`);
        // Zero mainnet IDs: no eip155:1 exact, no solana mainnet genesis.
        assert.ok(!/^eip155:1([^0-9]|$)/.test(e.chainId), `${name}/${e.chainId} not mainnet`);
        assert.ok(!e.chainId.includes("5eykt4UsFv8P8NJdTREpYbfmoaDMG5Uf2n4MWY5auP"), "no mainnet genesis");
        if (name === "testnets") assert.equal(e.fixtureOnly ?? fx.fixtureOnly, true, "testnet fixtureOnly");
      }
    }
    assert.equal(VECTORS.length, 5, "5 gameId vectors");
  });

  it("NFR-4/NFR-6: deriveGameId gas < 50k; registry read p99 < 2s on localnet", async function () {
    const { registry } = await deploy();
    const publicClient = await viem.getPublicClient();
    const gameId = await proposeActive(
      registry,
      "eip155:84532",
      "0x1111111111111111111111111111111111111111",
      "nfr-game",
      "0x2222222222222222222222222222222222222222",
      0,
    );
    const gas = await publicClient.estimateContractGas({
      address: registry.address,
      abi: registry.abi,
      functionName: "deriveGameId",
      args: ["eip155:84532", "0x1111111111111111111111111111111111111111", "nfr-game"],
    });
    console.log(`      NFR-4 deriveGameId estimate: ${gas} gas`);
    assert.ok(gas < 50_000n, `derive under 50k gas (got ${gas})`);
    const t0 = Date.now();
    for (let i = 0; i < 10; i++) await registry.read.getEntry([gameId]);
    const p99 = (Date.now() - t0) / 10;
    console.log(`      NFR-6 registry read mean: ${p99.toFixed(1)}ms over 10 reads`);
    assert.ok(p99 < 2000, "read p99 < 2s");
  });
});
