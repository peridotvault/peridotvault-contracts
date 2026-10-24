import { Connection, PublicKey } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const RPC = "http://127.0.0.1:8899";
const scratch = process.env.PAPERCLIP_RUN_SCRATCH_DIR;
const pid = new PublicKey(process.env.M2_PROGRAM_ID);
const attest = JSON.parse(readFileSync(`${scratch}/m2-attest.json`, "utf8"));
const h = (x) => Buffer.from(x.replace(/^0x/, ""), "hex");
const c = new Connection(RPC, "confirmed");

const [rec] = PublicKey.findProgramAddressSync(
  [Buffer.from("m1-recognition"), h(attest.message.gameId), h(attest.message.licensee),
   (() => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(attest.message.nonce)); return b; })()],
  pid);
console.log("rec PDA:", rec.toBase58());
const acc = await c.getAccountInfo(rec);
if (!acc) { console.log("recognition MISSING"); process.exit(2); }
const body = acc.data.subarray(8);
console.log("game:", `0x${body.subarray(0, 32).toString("hex")}`);
console.log("lic :", `0x${body.subarray(32, 64).toString("hex")}`);
console.log("digest:", `0x${body.subarray(64, 96).toString("hex")}`);
console.log("consumedAt:", body.readBigInt64LE(96).toString());
console.log("expect:", attest.digest);
const sigs = await c.getSignaturesForAddress(rec, { limit: 3 });
console.log("sigs:", JSON.stringify(sigs.map((s) => ({ sig: s.signature, slot: s.slot, err: s.err }))));
if (sigs.length > 0) {
  const tx = await c.getTransaction(sigs[0].signature, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  const logs = (tx.meta.logMessages || []).join("\n");
  const m = logs.match(/consumed (\d+) of/);
  console.log("cu:", m ? m[1] : "unknown");
  console.log("fee:", tx.meta.fee);
}
