import assert from "node:assert/strict";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { describe, it } from "node:test";
import {
  encodeAbiParameters,
  keccak256,
  pad,
  parseEventLogs,
  toHex,
} from "viem";

import { network } from "hardhat";

/**
 * ANT-487 M2 attestation round-trip (EVM side).
 *
 * Emit on home chain -> 3-of-5 ECDSA secp256k1 signatures over the EIP-712
 * PGL1-XCHAIN envelope -> verify + recognize on dest. Runs on the ephemeral
 * in-process Hardhat chain (0 mainnet txs, no secrets).
 *
 * Cross-stack note: the TS `manualDigest` below is an INDEPENDENT EIP-712
 * encoder (no contract calls). AC-VEC proves contract == independent encoder;
 * fixtures/xchain-attestation.vectors.json carries (domain, messages,
 * digests) so the Solana encoder (xchain_attest.rs) proves byte-identical
 * digests off the same file. Digest equality across three implementations
 * (Solidity / TS / Rust) is the transport-agnostic envelope proof.
 */

const ATTESTATION_TYPESTRING =
  "Attestation(bytes32 gameId,string homeChainId,string destChainId,bytes32 licensee,bool isFree,uint256 paidAmount,bytes paidToken,uint8 paidTokenFormat,uint8 receiptVersion,uint8 version,uint256 nonce,uint256 homeFinalizedBlock,uint64 observedAt,bytes32 transportId)";
const DOMAIN_TYPESTRING =
  "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

const VECTORS_PATH = new URL(
  "../../../fixtures/xchain-attestation.vectors.json",
  import.meta.url,
);

type Domain = {
  name: string;
  version: string;
  chainId: number;
  verifyingContract: `0x${string}`;
};

type AttMsg = {
  gameId: `0x${string}`;
  homeChainId: string;
  destChainId: string;
  licensee: `0x${string}`;
  isFree: boolean;
  paidAmount: bigint;
  paidToken: `0x${string}`;
  paidTokenFormat: number;
  receiptVersion: number;
  version: number;
  nonce: bigint;
  homeFinalizedBlock: bigint;
  observedAt: bigint;
  transportId: `0x${string}`;
};

const TYPES = {
  Attestation: [
    { name: "gameId", type: "bytes32" },
    { name: "homeChainId", type: "string" },
    { name: "destChainId", type: "string" },
    { name: "licensee", type: "bytes32" },
    { name: "isFree", type: "bool" },
    { name: "paidAmount", type: "uint256" },
    { name: "paidToken", type: "bytes" },
    { name: "paidTokenFormat", type: "uint8" },
    { name: "receiptVersion", type: "uint8" },
    { name: "version", type: "uint8" },
    { name: "nonce", type: "uint256" },
    { name: "homeFinalizedBlock", type: "uint256" },
    { name: "observedAt", type: "uint64" },
    { name: "transportId", type: "bytes32" },
  ],
} as const;

/** Independent EIP-712 digest (mirrors the contract, uses no contract). */
function manualDigest(domain: Domain, m: AttMsg): `0x${string}` {
  const structHash = keccak256(
    encodeAbiParameters(
      [
        { type: "bytes32" },
        { type: "bytes32" },
        { type: "bytes32" },
        { type: "bytes32" },
        { type: "bytes32" },
        { type: "bool" },
        { type: "uint256" },
        { type: "bytes32" },
        { type: "uint8" },
        { type: "uint8" },
        { type: "uint8" },
        { type: "uint256" },
        { type: "uint256" },
        { type: "uint64" },
        { type: "bytes32" },
      ],
      [
        keccak256(toHex(ATTESTATION_TYPESTRING)),
        m.gameId,
        keccak256(toHex(m.homeChainId)),
        keccak256(toHex(m.destChainId)),
        m.licensee,
        m.isFree,
        m.paidAmount,
        keccak256(m.paidToken),
        m.paidTokenFormat,
        m.receiptVersion,
        m.version,
        m.nonce,
        m.homeFinalizedBlock,
        m.observedAt,
        m.transportId,
      ],
    ),
  );
  const domainSep = keccak256(
    encodeAbiParameters(
      [
        { type: "bytes32" },
        { type: "bytes32" },
        { type: "bytes32" },
        { type: "uint256" },
        { type: "address" },
      ],
      [
        keccak256(toHex(DOMAIN_TYPESTRING)),
        keccak256(toHex(domain.name)),
        keccak256(toHex(domain.version)),
        BigInt(domain.chainId),
        domain.verifyingContract,
      ],
    ),
  );
  return keccak256(`0x1901${domainSep.slice(2)}${structHash.slice(2)}` as `0x${string}`);
}

function msgToJson(m: AttMsg): Record<string, unknown> {
  return {
    ...m,
    paidAmount: m.paidAmount.toString(),
    nonce: m.nonce.toString(),
    homeFinalizedBlock: m.homeFinalizedBlock.toString(),
    observedAt: m.observedAt.toString(),
  };
}

function msgFromJson(j: any): AttMsg {
  return {
    ...j,
    paidAmount: BigInt(j.paidAmount),
    nonce: BigInt(j.nonce),
    homeFinalizedBlock: BigInt(j.homeFinalizedBlock),
    observedAt: BigInt(j.observedAt),
  };
}

describe("M2 attestation round-trip (EVM)", async function () {
  const { viem, provider } = await network.connect();
  const wallets = await viem.getWalletClients();
  const publicClient = await viem.getPublicClient();
  const chainId = await publicClient.getChainId();

  // Attester set = accounts 10..14 (owner/deployer is account 0).
  const attesters = wallets.slice(10, 15);
  const attesterAddrs = attesters.map((w) => w.account.address) as `0x${string}`[];
  const outsider = wallets[15];

  async function deployAttester() {
    const attester = await viem.deployContract("XChainAttester", [attesterAddrs]);
    const [owner] = wallets;
    return { attester, owner };
  }

  async function signMsg(signer: any, domain: Domain, message: AttMsg) {
    return signer.signTypedData({
      domain: {
        name: domain.name,
        version: domain.version,
        chainId: domain.chainId,
        verifyingContract: domain.verifyingContract,
      },
      types: TYPES,
      primaryType: "Attestation",
      message: {
        ...message,
        paidToken: message.paidToken as `0x${string}`,
      },
    });
  }

  async function currentTime(): Promise<bigint> {
    const b = await publicClient.getBlock();
    return b.timestamp;
  }

  function paidMessage(ts: bigint): AttMsg {
    return {
      gameId: keccak256(toHex("m2-paid-game")),
      homeChainId: "eip155:84532",
      destChainId: "eip155:4202",
      licensee: pad("0x1111111111111111111111111111111111111111", { size: 32 }),
      isFree: false,
      paidAmount: 100n,
      paidToken: "0x2222222222222222222222222222222222222222",
      paidTokenFormat: 0,
      receiptVersion: 1,
      version: 1,
      nonce: 7n,
      homeFinalizedBlock: 12345678n,
      observedAt: ts,
      transportId: keccak256(toHex("wormhole")),
    };
  }

  function freeMessage(ts: bigint): AttMsg {
    return {
      gameId: keccak256(toHex("m2-free-game")),
      homeChainId: "eip155:84532",
      destChainId: "solana:devnet",
      // Solana-flavored licensee: full 32-byte pubkey-shaped value.
      licensee: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      isFree: true,
      paidAmount: 0n,
      paidToken: "0x",
      paidTokenFormat: 0,
      receiptVersion: 1,
      version: 1,
      nonce: 1n,
      homeFinalizedBlock: 12345679n,
      observedAt: ts,
      transportId: keccak256(toHex("layerzero-v2")),
    };
  }

  async function quorumSigs(domain: Domain, message: AttMsg, idxs = [0, 1, 2]) {
    const out: `0x${string}`[] = [];
    for (const i of idxs) out.push(await signMsg(attesters[i], domain, message));
    return out;
  }

  it("AC-VEC: independent TS encoder reproduces every committed fixture digest", async function () {
    assert.ok(existsSync(VECTORS_PATH), "attestation vectors fixture exists");
    const fx = JSON.parse(readFileSync(VECTORS_PATH, "utf8"));
    assert.equal(fx.typeString, ATTESTATION_TYPESTRING, "typestring pinned");
    assert.equal(fx.domain.name, "PGL1-XCHAIN");
    assert.equal(fx.domain.version, "1");
    assert.ok(fx.vectors.length >= 2, "paid + free vectors present");
    for (const v of fx.vectors) {
      const got = manualDigest(fx.domain, msgFromJson(v.message));
      assert.equal(got.toLowerCase(), v.digest.toLowerCase(), `vector ${v.label}`);
    }
  });

  it("AC-VEC2: contract hashAttestation == independent encoder (paid + free)", async function () {
    const { attester } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const ts = await currentTime();
    for (const m of [paidMessage(ts), freeMessage(ts)]) {
      const onchain = (await attester.read.hashAttestation([msgToViem(m)])) as string;
      assert.equal(onchain.toLowerCase(), manualDigest(domain, m).toLowerCase(), "contract==manual");
    }
    // WRITE_VECTORS=1 regenerates the committed cross-stack fixture.
    if (process.env.WRITE_VECTORS === "1") {
      const vectors = [
        { label: "paid-evm-licensee", message: msgToJson(paidMessage(ts)) },
        { label: "free-solana-licensee", message: msgToJson(freeMessage(ts)) },
      ].map((v) => ({ ...v, digest: manualDigest(domain, msgFromJson(v.message)) }));
      writeFileSync(
        VECTORS_PATH,
        JSON.stringify(
          { domain, typeString: ATTESTATION_TYPESTRING, vectors },
          null,
          2,
        ) + "\n",
      );
      console.log(`      vectors written for ${attester.address}`);
    }
  });

  it("AC-ROUNDTRIP: paid envelope 3-of-5 -> recognized + event", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    const sigs = await quorumSigs(domain, m);
    const hash = await attester.write.consume([msgToViem(m), sigs]);
    assert.equal(await attester.read.isRecognized([m.gameId, m.licensee]), true);
    assert.equal(await attester.read.consumed([manualDigest(domain, m)]), true);
    const receipt = await publicClient.getTransactionReceipt({ hash });
    const events = parseEventLogs({ abi: attester.abi, logs: receipt.logs });
    const ev = events.find((x) => x.eventName === "AttestationConsumed");
    assert.ok(ev, "AttestationConsumed emitted");
    assert.equal((ev!.args as any).nonce, m.nonce);
  });

  it("AC-ROUNDTRIP-FREE: free envelope (solana licensee, LZ transport) round-trips", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = freeMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    await attester.write.consume([msgToViem(m), await quorumSigs(domain, m, [2, 3, 4])]);
    assert.equal(await attester.read.isRecognized([m.gameId, m.licensee]), true);
  });

  it("AC-REPLAY: same digest twice reverts AlreadyConsumed", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    const sigs = await quorumSigs(domain, m);
    await attester.write.consume([msgToViem(m), sigs]);
    await assert.rejects(() => attester.write.consume([msgToViem(m), sigs]), /AlreadyConsumed/);
    // Same licensee, fresh nonce -> fresh digest -> consumes fine.
    const m2 = { ...m, nonce: m.nonce + 1n };
    await attester.write.consume([msgToViem(m2), await quorumSigs(domain, m2)]);
    assert.equal(await attester.read.isRecognized([m2.gameId, m2.licensee]), true);
  });

  it("AC-THRESHOLD: 2-of-5, unknown signer, and double-counted signer all fail", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    // 2 signatures < threshold.
    await assert.rejects(
      async () => attester.write.consume([msgToViem(m), await quorumSigs(domain, m, [0, 1])]),
      /InsufficientSignatures/,
    );
    // Outsider signature fails closed (unknown, not silently skipped).
    const outsiderSig = await signMsg(outsider, domain, m);
    const two = await quorumSigs(domain, m, [0, 1]);
    await assert.rejects(
      () => attester.write.consume([msgToViem(m), [...two, outsiderSig]]),
      /UnknownSigner/,
    );
    // Same member twice + one other = only 2 distinct -> DuplicateSignature.
    const dup0 = await signMsg(attesters[0], domain, m);
    const one = await quorumSigs(domain, m, [1]);
    await assert.rejects(
      () => attester.write.consume([msgToViem(m), [dup0, ...one, dup0]]),
      /DuplicateSignature/,
    );
    // Sanity: the honest quorum still works after the sad paths.
    await attester.write.consume([msgToViem(m), await quorumSigs(domain, m)]);
    assert.equal(await attester.read.isRecognized([m.gameId, m.licensee]), true);
  });

  it("AC-PAYBIND: tampered isFree/paidAmount breaks quorum; invalid envelope fails even if signed", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    const sigs = await quorumSigs(domain, m);
    // Flip the nonce post-sign: envelope stays payment-valid, but the digest
    // changes -> recoveries miss the set (quorum binding proof).
    const tampered = { ...m, nonce: m.nonce + 1n };
    await assert.rejects(
      () => attester.write.consume([msgToViem(tampered), sigs]),
      /UnknownSigner/,
    );
    // Flip paid->free post-sign with the token still attached: the dest
    // payment rule fires before signature recovery (cheap check first).
    const flipped = { ...m, isFree: true, paidAmount: 0n };
    await assert.rejects(
      () => attester.write.consume([msgToViem(flipped), sigs]),
      /PaidTokenNonEmptyOnFreePath/,
    );
    // Quorum signs a free-with-amount envelope: dest rule fires regardless.
    const bad = { ...freeMessage(await currentTime()), paidAmount: 5n };
    const badSigs = await quorumSigs(domain, bad);
    await assert.rejects(
      () => attester.write.consume([msgToViem(bad), badSigs]),
      /PaidAmountNonZeroOnFreePath/,
    );
  });

  it("AC-TRANSPORT: transportId bound into digest; transports swappable (no lock-in)", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    const sigs = await quorumSigs(domain, m); // signed over wormhole label
    const relabeled = { ...m, transportId: keccak256(toHex("layerzero-v2")) };
    await assert.rejects(
      () => attester.write.consume([msgToViem(relabeled), sigs]),
      /UnknownSigner/,
      "transport swap without re-sign fails",
    );
    // Re-signed under the new label: same contract, no code change.
    await attester.write.consume([msgToViem(relabeled), await quorumSigs(domain, relabeled)]);
    assert.equal(await attester.read.isRecognized([relabeled.gameId, relabeled.licensee]), true);
  });

  it("AC-VERSION: envelope version skew and future versions fail closed", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    const skew = { ...m, receiptVersion: 0 };
    await assert.rejects(
      async () => attester.write.consume([msgToViem(skew), await quorumSigs(domain, skew)]),
      /VersionMismatch/,
    );
    const future = { ...m, version: 2, receiptVersion: 2 };
    await assert.rejects(
      async () => attester.write.consume([msgToViem(future), await quorumSigs(domain, future)]),
      /UnsupportedEnvelopeVersion/,
    );
  });

  it("AC-REORG: unconfigured home fails closed; delay enforced; warp releases", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    // No delay configured yet -> fail closed.
    await assert.rejects(
      async () => attester.write.consume([msgToViem(m), await quorumSigs(domain, m)]),
      /ReorgDelayUnconfigured/,
    );
    await attester.write.setReorgDelay(["eip155:84532", 3600n], { account: owner.account });
    // Fresh observation -> inside the reorg window.
    const fresh = { ...m, observedAt: await currentTime() };
    await assert.rejects(
      async () => attester.write.consume([msgToViem(fresh), await quorumSigs(domain, fresh)]),
      /ReorgDelayNotElapsed/,
    );
    // Future observedAt is a corrupt attestation, not an early one.
    const future = { ...m, observedAt: (await currentTime()) + 10_000n };
    await assert.rejects(
      async () => attester.write.consume([msgToViem(future), await quorumSigs(domain, future)]),
      /ObservedAtInFuture/,
    );
    // Warp past the window -> consumes.
    await provider.request({ method: "evm_increaseTime", params: [3601] });
    await provider.request({ method: "evm_mine", params: [] });
    await attester.write.consume([msgToViem(fresh), await quorumSigs(domain, fresh)]);
    assert.equal(await attester.read.isRecognized([fresh.gameId, fresh.licensee]), true);
  });

  it("AC-GOV: rotation installs new set, clears stale keys; bad sets rejected", async function () {
    const { attester, owner } = await deployAttester();
    await assert.rejects(
      () =>
        attester.write.rotateSigners([
          [
            attesterAddrs[0],
            attesterAddrs[0],
            attesterAddrs[1],
            attesterAddrs[2],
            attesterAddrs[3],
          ],
        ]),
      /InvalidSignerSet/,
      "duplicate member rejected",
    );
    const next = wallets.slice(15, 20).map((w) => w.account.address) as `0x${string}`[];
    await attester.write.rotateSigners([next], { account: owner.account });
    // Old quorum no longer recognized...
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    const oldSigs = await quorumSigs(domain, m);
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    await assert.rejects(
      () => attester.write.consume([msgToViem(m), oldSigs]),
      /UnknownSigner/,
    );
    // ...new quorum works.
    const newSigs: `0x${string}`[] = [];
    for (const w of wallets.slice(15, 18)) newSigs.push(await signMsg(w, domain, m));
    await attester.write.consume([msgToViem(m), newSigs]);
    assert.equal(await attester.read.isRecognized([m.gameId, m.licensee]), true);
  });

  it("NFR-GAS: consume (3 sigs) gas measured and bounded", async function () {
    const { attester, owner } = await deployAttester();
    const domain: Domain = {
      name: "PGL1-XCHAIN",
      version: "1",
      chainId,
      verifyingContract: attester.address,
    };
    const m = paidMessage(await currentTime());
    await attester.write.setReorgDelay(["eip155:84532", 0n], { account: owner.account });
    const gas = await publicClient.estimateContractGas({
      address: attester.address,
      abi: attester.abi,
      functionName: "consume",
      args: [msgToViem(m), await quorumSigs(domain, m)],
    });
    console.log(`      NFR consume(3-of-5) estimate: ${gas} gas`);
    assert.ok(gas < 400_000n, `consume under 400k gas (got ${gas})`);
  });
});

/** viem contract-call shape: BigInts for all uint fields incl. uint8/uint64. */
function msgToViem(m: AttMsg): any {
  return {
    gameId: m.gameId,
    homeChainId: m.homeChainId,
    destChainId: m.destChainId,
    licensee: m.licensee,
    isFree: m.isFree,
    paidAmount: m.paidAmount,
    paidToken: m.paidToken,
    paidTokenFormat: m.paidTokenFormat,
    receiptVersion: m.receiptVersion,
    version: m.version,
    nonce: m.nonce,
    homeFinalizedBlock: m.homeFinalizedBlock,
    observedAt: m.observedAt,
    transportId: m.transportId,
  };
}
