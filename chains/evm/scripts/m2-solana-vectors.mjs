/** M2 Solana-dest attestation material builder (ANT-487).
 *
 * Usage:
 *   node chains/evm/scripts/m2-solana-vectors.mjs   # protocol crate::ID domain
 *   M2_DECLARED_PROGRAM_ID=<temp-base58> M2_OBSERVED_AT=<chain-unix> node chains/evm/scripts/m2-solana-vectors.mjs
 * Writes $M2_OUT_DIR/m2-attest.json (fallback $PAPERCLIP_RUN_SCRATCH_DIR).
 * Pair with chains/solana/scripts/m2-solana-vectors.mjs for localnet submit.
 */
/**
 * ANT-487 M2: build Solana-dest attestation material for localnet consume.
 *
 * TEST-ONLY keys (0x00..01 .. 0x00..05): never funded on any real network,
 * used solely to sign localnet attestations. Runs with NO RPC (pure local
 * signing via viem) and writes attest.json into the run scratch dir.
 *
 * The Solana-dest EIP-712 domain is DERIVED exactly like
 * xchain_consume.rs: chain_id = u64(be(keccak(dest_chain_id)[..8])),
 * verifying_contract = keccak(registry_program_id_bytes)[..20], where the
 * program id is the DECLARED protocol constant (crate::ID), not the
 * localnet deploy address.
 */
import { keccak256, encodeAbiParameters, toHex } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { writeFileSync } from "node:fs";

const ATTESTATION_TYPESTRING =
  "Attestation(bytes32 gameId,string homeChainId,string destChainId,bytes32 licensee,bool isFree,uint256 paidAmount,bytes paidToken,uint8 paidTokenFormat,uint8 receiptVersion,uint8 version,uint256 nonce,uint256 homeFinalizedBlock,uint64 observedAt,bytes32 transportId)";
const DOMAIN_TYPESTRING =
  "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

// Declared protocol program id (crate::ID in registry/src/lib.rs).
const DECLARED_PROGRAM_ID = process.env.M2_DECLARED_PROGRAM_ID || "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE";
const DEST_CHAIN_ID = process.env.M2_DEST_CHAIN || "solana:localnet";

const ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
function base58Decode(s) {
  let n = 0n;
  for (const c of s) n = n * 58n + BigInt(ALPHABET.indexOf(c));
  let hex = n.toString(16).padStart(64, "0");
  let leading = 0;
  for (const c of s) {
    if (c !== "1") break;
    leading++;
  }
  return Buffer.concat([Buffer.alloc(leading), Buffer.from(hex, "hex")]);
}

function manualDigest(domain, m) {
  const structHash = keccak256(
    encodeAbiParameters(
      [
        { type: "bytes32" }, { type: "bytes32" }, { type: "bytes32" },
        { type: "bytes32" }, { type: "bytes32" }, { type: "bool" },
        { type: "uint256" }, { type: "bytes32" }, { type: "uint8" },
        { type: "uint8" }, { type: "uint8" }, { type: "uint256" },
        { type: "uint256" }, { type: "uint64" }, { type: "bytes32" },
      ],
      [
        keccak256(toHex(ATTESTATION_TYPESTRING)),
        m.gameId, keccak256(toHex(m.homeChainId)), keccak256(toHex(m.destChainId)),
        m.licensee, m.isFree, m.paidAmount, keccak256(m.paidToken),
        m.paidTokenFormat, m.receiptVersion, m.version, m.nonce,
        m.homeFinalizedBlock, m.observedAt, m.transportId,
      ],
    ),
  );
  const domainSep = keccak256(
    encodeAbiParameters(
      [{ type: "bytes32" }, { type: "bytes32" }, { type: "bytes32" }, { type: "uint256" }, { type: "address" }],
      [keccak256(toHex(DOMAIN_TYPESTRING)), keccak256(toHex(domain.name)), keccak256(toHex(domain.version)), domain.chainId, domain.verifyingContract],
    ),
  );
  return keccak256(`0x1901${domainSep.slice(2)}${structHash.slice(2)}`);
}

const TYPES = {
  Attestation: [
    { name: "gameId", type: "bytes32" }, { name: "homeChainId", type: "string" },
    { name: "destChainId", type: "string" }, { name: "licensee", type: "bytes32" },
    { name: "isFree", type: "bool" }, { name: "paidAmount", type: "uint256" },
    { name: "paidToken", type: "bytes" }, { name: "paidTokenFormat", type: "uint8" },
    { name: "receiptVersion", type: "uint8" }, { name: "version", type: "uint8" },
    { name: "nonce", type: "uint256" }, { name: "homeFinalizedBlock", type: "uint256" },
    { name: "observedAt", type: "uint64" }, { name: "transportId", type: "bytes32" },
  ],
};

const programBytes = base58Decode(DECLARED_PROGRAM_ID);
const destTag = Buffer.from(keccak256(toHex(DEST_CHAIN_ID)).slice(2), "hex");
const verifier = `0x${Buffer.from(keccak256(`0x${programBytes.toString("hex")}`).slice(2), "hex").subarray(0, 20).toString("hex")}`;
const domain = {
  name: "PGL1-XCHAIN",
  version: "1",
  chainId: destTag.readBigUInt64BE(0).toString(),
  verifyingContract: verifier,
};

// Fixed TEST-ONLY attester keys.
const keys = [1, 2, 3, 4, 5].map((i) => `0x${i.toString(16).padStart(64, "0")}`);
const accounts = keys.map((k) => privateKeyToAccount(k));
const signers = accounts.map((a) => a.address.toLowerCase());
console.log("attester addrs:", signers);

const nowSec = Number(process.env.M2_OBSERVED_AT || Math.floor(Date.now() / 1000));
const paidToken = "0x2222222222222222222222222222222222222222";
const message = {
  gameId: keccak256(toHex("m2-solana-paid")),
  homeChainId: "eip155:84532",
  destChainId: DEST_CHAIN_ID,
  licensee: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  isFree: false,
  paidAmount: 100n,
  paidToken,
  paidTokenFormat: 0,
  receiptVersion: 1,
  version: 1,
  nonce: process.env.M2_NONCE ? BigInt(process.env.M2_NONCE) : 7n,
  homeFinalizedBlock: 12345678n,
  observedAt: BigInt(nowSec),
  transportId: keccak256(toHex("wormhole")),
};

const digest = manualDigest(domain, message);
console.log("solana-domain digest:", digest);

// Sign the RAW EIP-712 digest (manualDigest output). signTypedData is NOT
// used: viem numeric chainId would lose precision on the u64 domain tag,
// while raw signing binds exactly the bytes the program recomputes.
const sigs = [];
for (const a of accounts.slice(0, 3)) {
  sigs.push(await a.sign({ hash: digest }));
}
console.log("sigs:", sigs.length);

const out = {
  domain,
  message: { ...message, paidAmount: "100", nonce: message.nonce.toString(), homeFinalizedBlock: "12345678", observedAt: message.observedAt.toString() },
  digest,
  signatures: sigs,
  signers,
};
const outPath = process.env.M2_OUT || `${process.env.M2_OUT_DIR || process.env.PAPERCLIP_RUN_SCRATCH_DIR}/m2-attest.json`;
writeFileSync(outPath, JSON.stringify(out, null, 2));
console.log("wrote", outPath);
