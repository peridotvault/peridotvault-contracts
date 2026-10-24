/** M2 Solana localnet consume submitter (ANT-487).
 *
 * Usage (test validator ONLY — never mainnet):
 *   M2_PROGRAM_ID=<deployed-id> PAPERCLIP_RUN_SCRATCH_DIR=<scratch> node chains/solana/scripts/m2-localnet-consume.mjs
 * Reads <scratch>/m2-attest.json (built by chains/evm/scripts/m2-solana-vectors.mjs).
 * Prints EVIDENCE lines for docs/finality-measurements.md.
 */
/** M2 Solana localnet consume round-trip (ANT-487).

Usage (test validator only — never mainnet):
  solana-test-validator --reset &
  solana program deploy --url http://127.0.0.1:8899 \
    --keypair <payer>.json --program-id <program>.json \
    target/sbpf-solana-solana/release/registry.so
  node chains/evm/scripts/m2-solana-vectors.mjs   # needs M2_* env for temp ids
  M2_PROGRAM_ID=<deployed-id> PAPERCLIP_RUN_SCRATCH_DIR=<scratch> \
    node chains/solana/scripts/m2-localnet-consume.mjs

Asserts: recognition record matches, replay rejected, tampered-amount
rejected. Prints EVIDENCE lines for docs/finality-measurements.md.
 *
 * ANT-487 M2: Solana localnet consume round-trip (test validator only).
 *
 * Reads attest.json (Script A output) + deploys nothing (program already
 * deployed to localnet). Uses throwaway localnet keypairs from the run
 * scratch dir. Prints evidence lines consumed by docs/finality-measurements.md
 * and the ANT-487 task comment.
 */
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction,
  ComputeBudgetProgram, sendAndConfirmTransaction,
} from "@solana/web3.js";

const RPC = "http://127.0.0.1:8899";
const PROGRAM_ID = new PublicKey(process.env.M2_PROGRAM_ID);
const connection = new Connection(RPC, "confirmed");
const scratch = process.env.PAPERCLIP_RUN_SCRATCH_DIR;
const payer = Keypair.fromSecretKey(Buffer.from(JSON.parse(readFileSync(`${scratch}/m2-payer-keypair.json`, "utf8"))));

const disc = (name) => createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
const h = (x) => Buffer.from(x.replace(/^0x/, ""), "hex");
const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
const u64 = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const i64 = (n) => { const b = Buffer.alloc(8); b.writeBigInt64LE(BigInt(n)); return b; };
const borshStr = (s) => Buffer.concat([u32(Buffer.byteLength(s)), Buffer.from(s, "utf8")]);
const borshBytes = (buf) => Buffer.concat([u32(buf.length), buf]);

function encodeConsume(a, sigs65) {
  const parts = [disc("consume_xchain_attestation")];
  parts.push(h(a.gameId), borshStr(a.homeChainId), borshStr(a.destChainId), h(a.licensee));
  parts.push(Buffer.from([a.isFree ? 1 : 0]), u64(a.paidAmount), borshBytes(h(a.paidToken)));
  parts.push(Buffer.from([a.paidTokenFormat, a.receiptVersion, a.version]));
  parts.push(u64(a.nonce), u64(a.homeFinalizedBlock), u64(a.observedAt), h(a.transportId));
  parts.push(u32(sigs65.length));
  for (const s of sigs65) {
    const raw = h(s);
    parts.push(raw.subarray(0, 64), Buffer.from([raw[64] - 27]));
  }
  return Buffer.concat(parts);
}

const attest = JSON.parse(readFileSync(`${scratch}/m2-attest.json`, "utf8"));
const A = attest.message;
const gameId = h(A.gameId), licensee = h(A.licensee);

const [configPda] = PublicKey.findProgramAddressSync([Buffer.from("m1-attester")], PROGRAM_ID);
console.log("config PDA:", configPda.toBase58());

// 1. init attester config (signers from attest.json, reorg_delay 2s).
// Idempotent: skip if the config PDA already exists (fresh validator reruns).
{
  const existing = await connection.getAccountInfo(configPda, "confirmed");
  const discr = disc("init_xchain_attester");
  const updateDisc = disc("update_xchain_attester");
  const signersBuf = Buffer.concat(attest.signers.map((s) => h(s)));
  const data = Buffer.concat([(existing ? updateDisc : discr), signersBuf, i64(2)]);
  const keys = [
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: configPda, isSigner: false, isWritable: true },
  ];
  if (!existing) keys.push({ pubkey: SystemProgram.programId, isSigner: false, isWritable: false });
  const ix = new TransactionInstruction({ keys, programId: PROGRAM_ID, data });
  const sig = await sendAndConfirmTransaction(connection, new Transaction().add(ix), [payer], { commitment: "confirmed" });
  console.log(existing ? "update_xchain_attester tx:" : "init_xchain_attester tx:", sig);
}

// 2. consume (paid, wormhole label) with 3-of-5.
const [recPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("m1-recognition"), gameId, licensee, u64(A.nonce)], PROGRAM_ID);
console.log("recognition PDA:", recPda.toBase58());
const slotBefore = await connection.getSlot("processed");
const t0 = Date.now();
let consumeSig;
{
  const data = encodeConsume({ ...A, paidToken: A.paidToken }, attest.signatures);
  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: configPda, isSigner: false, isWritable: false },
      { pubkey: recPda, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID, data,
  });
  consumeSig = await sendAndConfirmTransaction(connection, new Transaction().add(
    ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 }),
    ix,
  ), [payer], { commitment: "finalized" });
}
const finalizedMs = Date.now() - t0;
const finSlot = (await connection.getTransaction(consumeSig, { commitment: "finalized" })).slot;
console.log("consume tx:", consumeSig);
console.log(`EVIDENCE localnet consume: slot=${finSlot} slots_elapsed~${finSlot - slotBefore} time_to_finalized_ms=${finalizedMs}`);

// 3. assert recognition record.
{
  const acc = await connection.getAccountInfo(recPda, "finalized");
  if (!acc) throw new Error("recognition account missing");
  const body = acc.data.subarray(8);
  const gotGame = `0x${body.subarray(0, 32).toString("hex")}`;
  const gotLic = `0x${body.subarray(32, 64).toString("hex")}`;
  const gotDigest = `0x${body.subarray(64, 96).toString("hex")}`;
  const consumedAt = body.readBigInt64LE(96);
  console.log("recognition:", { gotGame, gotLic, gotDigest, consumedAt: consumedAt.toString() });
  if (gotGame !== A.gameId || gotLic !== A.licensee || gotDigest !== attest.digest) throw new Error("MISMATCH");
  console.log("EVIDENCE recognition matches attestation (gameId, licensee, digest)");
}

// 4. replay: same args -> PDA init fails.
{
  const data = encodeConsume({ ...A, paidToken: A.paidToken }, attest.signatures);
  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: configPda, isSigner: false, isWritable: false },
      { pubkey: recPda, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID, data,
  });
  let err = null;
  try {
    await sendAndConfirmTransaction(connection, new Transaction().add(ix), [payer], { commitment: "confirmed" });
  } catch (e) { err = String(e).slice(0, 300); }
  console.log("EVIDENCE replay rejected:", err ? "yes" : "NO (BUG)");
  if (err) console.log("  err:", err.split("\n")[0]);
  if (!err) throw new Error("replay was NOT rejected");
}

// 5. tampered paidAmount (no re-sign) -> UnknownAttester.
{
  const tampered = { ...A, paidAmount: "999" };
  const [recPda2] = PublicKey.findProgramAddressSync(
    [Buffer.from("m1-recognition"), gameId, licensee, u64(9999)], PROGRAM_ID);
  void recPda2;
  // NOTE: nonce must differ to reach signature recovery (fresh PDA); use nonce 8.
  const t2 = { ...A, paidAmount: "999", nonce: "8" };
  const [recPda3] = PublicKey.findProgramAddressSync(
    [Buffer.from("m1-recognition"), gameId, licensee, u64(8)], PROGRAM_ID);
  const data = encodeConsume({ ...t2, paidToken: t2.paidToken }, attest.signatures);
  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: configPda, isSigner: false, isWritable: false },
      { pubkey: recPda3, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID, data,
  });
  let err = null;
  try {
    await sendAndConfirmTransaction(connection, new Transaction().add(ix), [payer], { commitment: "confirmed" });
  } catch (e) { err = String(e).slice(0, 300); }
  console.log("EVIDENCE tampered-amount rejected:", err ? "yes" : "NO (BUG)");
  if (err) console.log("  err:", err.split("\n")[0]);
  if (!err) throw new Error("tamper was NOT rejected");
}
console.log("ALL SOLANA LOCALNET CHECKS PASSED");
