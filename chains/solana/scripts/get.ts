import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import * as os from "os";
import * as path from "path";

// # Localnet (default)
// npx ts-node scripts/get.ts
// # Devnet
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/get.ts

// ============================================================
//  Peridot Vault — Get All State
// ============================================================
//  Fetches and prints configuration + linked state for all
//  three programs (pgl1, registry, game-store).
//
//  Usage:
//    npx ts-node scripts/get.ts
// ============================================================

declare const require: any;

function derivePda(seeds: Buffer[], programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

async function checkConnection(connection: anchor.web3.Connection): Promise<void> {
  try {
    await connection.getSlot();
  } catch {
    throw new Error(
      `Unable to reach RPC at ${connection.rpcEndpoint}.\n` +
        `If you meant to use Devnet, run with:\n` +
        `  ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/get.ts ...`,
    );
  }
}

async function accountExists(connection: anchor.web3.Connection, address: PublicKey): Promise<boolean> {
  return (await connection.getAccountInfo(address)) !== null;
}

function fmtPubkey(pk: PublicKey | string | undefined): string {
  if (!pk) return "N/A";
  return typeof pk === "string" ? pk : pk.toBase58();
}

function fmtBool(val: boolean | undefined): string {
  return val === undefined ? "N/A" : val ? "true" : "false";
}

function fmtBn(val: any): string {
  if (val === undefined || val === null) return "N/A";
  if (typeof val === "string") return val;
  if (typeof val === "number") return String(val);
  if (val.toString) return val.toString();
  return String(val);
}

function fmtDate(ts: number | undefined): string {
  if (ts === undefined) return "N/A";
  return new Date(ts * 1000).toISOString();
}

async function main() {
  process.env.ANCHOR_PROVIDER_URL ||= "http://127.0.0.1:8899";
  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  await checkConnection(provider.connection);

  const wallet = provider.wallet as anchor.Wallet & { payer: anchor.web3.Keypair };
  const authority = wallet.payer;

  console.log("\n============================================================");
  console.log("  Peridot Vault — Get All State");
  console.log("============================================================");
  console.log("  Network :", provider.connection.rpcEndpoint);
  console.log("  Wallet  :", authority.publicKey.toBase58());

  const pglIdl = require("../target/idl/pgl1.json");
  const registryIdl = require("../target/idl/registry.json");
  const storeIdl = require("../target/idl/game_store.json");

  const pglProgram = new Program(pglIdl, provider) as any;
  const registryProgram = new Program(registryIdl, provider) as any;
  const storeProgram = new Program(storeIdl, provider) as any;

  // --- Configs -----------------------------------------------------------
  const pglConfigPda = derivePda([Buffer.from("pgl_config")], pglProgram.programId);
  const registryConfigPda = derivePda([Buffer.from("registry_config")], registryProgram.programId);
  const storeConfigPda = derivePda([Buffer.from("store_config")], storeProgram.programId);

  console.log("\n--- PGL1 Config ---");
  if (await accountExists(provider.connection, pglConfigPda)) {
    const c = (await pglProgram.account.pglConfig.fetch(pglConfigPda)) as any;
    console.log("  PDA                :", pglConfigPda.toBase58());
    console.log("  Authority          :", fmtPubkey(c.authority));
    console.log("  Treasury           :", fmtPubkey(c.treasury));
    console.log("  CreateGameFee      :", fmtBn(c.createGameFeeLamports), "lamports");
    console.log("  Bump               :", c.bump);
  } else {
    console.log("  [NOT INITIALIZED]");
  }

  console.log("\n--- Registry Config ---");
  if (await accountExists(provider.connection, registryConfigPda)) {
    const c = (await registryProgram.account.registryConfig.fetch(registryConfigPda)) as any;
    console.log("  PDA                :", registryConfigPda.toBase58());
    console.log("  Authority          :", fmtPubkey(c.authority));
    console.log("  Treasury           :", fmtPubkey(c.treasury));
    console.log("  Pgl1Program        :", fmtPubkey(c.pgl1Program));
    console.log("  Bump               :", c.bump);
  } else {
    console.log("  [NOT INITIALIZED]");
  }

  console.log("\n--- Game Store Config ---");
  if (await accountExists(provider.connection, storeConfigPda)) {
    const c = (await storeProgram.account.storeConfig.fetch(storeConfigPda)) as any;
    console.log("  PDA                :", storeConfigPda.toBase58());
    console.log("  Authority          :", fmtPubkey(c.authority));
    console.log("  Treasury           :", fmtPubkey(c.treasury));
    console.log("  PlatformFeeBps     :", c.platformFeeBps);
    console.log("  DefaultReferralBps :", c.defaultReferralBps);
    console.log("  MaxReferralBps     :", c.maxReferralBps);
    console.log("  StoreActor         :", fmtPubkey(c.storeActor));
    console.log("  Bump               :", c.bump);
  } else {
    console.log("  [NOT INITIALIZED]");
  }

  // --- Authorized Programs (Store) ---------------------------------------
  console.log("\n--- Store Authorized Programs ---");
  const authPrograms = (await storeProgram.account.authorizedProgram.all()) as any[];
  if (authPrograms.length === 0) {
    console.log("  (none)");
  } else {
    authPrograms.forEach((a) => {
      console.log(
        `  ${a.publicKey.toBase58()} | program=${fmtPubkey(a.account.programId)} | role=${a.account.role} | active=${fmtBool(a.account.active)}`
      );
    });
  }

  // --- Authorized Actors (PGL1) ------------------------------------------
  console.log("\n--- PGL1 Authorized Actors ---");
  const authActors = (await pglProgram.account.authorizedActor.all()) as any[];
  if (authActors.length === 0) {
    console.log("  (none)");
  } else {
    authActors.forEach((a) => {
      console.log(
        `  ${a.publicKey.toBase58()} | actor=${fmtPubkey(a.account.actor)} | active=${fmtBool(a.account.active)}`
      );
    });
  }

  // --- Payment Tokens (Registry) ---------------------------------------
  console.log("\n--- Registry Accepted Payment Tokens ---");
  const regTokens = (await registryProgram.account.acceptedPaymentToken.all()) as any[];
  if (regTokens.length === 0) {
    console.log("  (none)");
  } else {
    regTokens.forEach((t) => {
      console.log(
        `  ${t.publicKey.toBase58()} | mint=${fmtPubkey(t.account.mint)} | active=${fmtBool(t.account.active)} | fee=${fmtBn(t.account.feeAmount)}`
      );
    });
  }

  // --- Payment Tokens (Store) --------------------------------------------
  console.log("\n--- Store Accepted Payment Tokens ---");
  const storeTokens = (await storeProgram.account.acceptedPaymentToken.all()) as any[];
  if (storeTokens.length === 0) {
    console.log("  (none)");
  } else {
    storeTokens.forEach((t) => {
      console.log(
        `  ${t.publicKey.toBase58()} | mint=${fmtPubkey(t.account.mint)} | active=${fmtBool(t.account.active)}`
      );
    });
  }

  // --- Publish Grants ----------------------------------------------------
  console.log("\n--- Registry Publish Grants ---");
  const grants = (await registryProgram.account.publishGrant.all()) as any[];
  console.log(`  Total: ${grants.length}`);
  grants.slice(0, 10).forEach((g) => {
    console.log(
      `  ${g.publicKey.toBase58()} | expiredAt=${g.account.expiredAt === null ? "never" : fmtDate(g.account.expiredAt)} | bump=${g.account.bump}`
    );
  });
  if (grants.length > 10) console.log(`  ... and ${grants.length - 10} more`);

  // --- Registry Games ----------------------------------------------------
  console.log("\n--- Registry Games ---");
  const regGames = (await registryProgram.account.registryGame.all()) as any[];
  console.log(`  Total: ${regGames.length}`);
  regGames.slice(0, 10).forEach((g) => {
    console.log(
      `  ${g.publicKey.toBase58()} | game=${fmtPubkey(g.account.game)} | gameId=${g.account.gameId} | status=${g.account.status} | registeredAt=${fmtDate(g.account.registeredAt)}`
    );
  });
  if (regGames.length > 10) console.log(`  ... and ${regGames.length - 10} more`);

  // --- PGL1 Games --------------------------------------------------------
  console.log("\n--- PGL1 Games ---");
  const games = (await pglProgram.account.game.all()) as any[];
  console.log(`  Total: ${games.length}`);
  games.slice(0, 10).forEach((g) => {
    console.log(
      `  ${g.publicKey.toBase58()} | creator=${fmtPubkey(g.account.creator)} | publisher=${fmtPubkey(g.account.publisher)} | gameId=${g.account.gameId} | nonce=${fmtBn(g.account.nonce)} | createdAt=${fmtDate(g.account.createdAt)}`
    );
  });
  if (games.length > 10) console.log(`  ... and ${games.length - 10} more`);

  // --- Game Store Configs ------------------------------------------------
  console.log("\n--- Game Store Configs ---");
  const storeConfigs = (await storeProgram.account.gameStoreConfig.all()) as any[];
  console.log(`  Total: ${storeConfigs.length}`);
  storeConfigs.slice(0, 10).forEach((s) => {
    console.log(
      `  ${s.publicKey.toBase58()} | game=${fmtPubkey(s.account.game)} | active=${fmtBool(s.account.active)} | referralBps=${s.account.referralBps ?? "default"} | discountBps=${s.account.discountBps ?? "none"}`
    );
  });
  if (storeConfigs.length > 10) console.log(`  ... and ${storeConfigs.length - 10} more`);

  // --- Game Payment Options ----------------------------------------------
  console.log("\n--- Game Payment Options ---");
  const paymentOptions = (await storeProgram.account.gamePaymentOption.all()) as any[];
  console.log(`  Total: ${paymentOptions.length}`);
  paymentOptions.slice(0, 10).forEach((p) => {
    console.log(
      `  ${p.publicKey.toBase58()} | game=${fmtPubkey(p.account.game)} | mint=${fmtPubkey(p.account.mint)} | basePrice=${fmtBn(p.account.basePrice)} | active=${fmtBool(p.account.active)}`
    );
  });
  if (paymentOptions.length > 10) console.log(`  ... and ${paymentOptions.length - 10} more`);

  // --- Purchase Receipts -------------------------------------------------
  console.log("\n--- Purchase Receipts ---");
  const receipts = (await storeProgram.account.purchaseReceipt.all()) as any[];
  console.log(`  Total: ${receipts.length}`);
  receipts.slice(0, 10).forEach((r) => {
    console.log(
      `  ${r.publicKey.toBase58()} | buyer=${fmtPubkey(r.account.buyer)} | game=${fmtPubkey(r.account.game)} | paid=${fmtBn(r.account.paidAmount)} | final=${fmtBn(r.account.finalPrice)} | referralBps=${r.account.referralBpsApplied} | purchasedAt=${fmtDate(r.account.purchasedAt)}`
    );
  });
  if (receipts.length > 10) console.log(`  ... and ${receipts.length - 10} more`);

  // --- Licenses ----------------------------------------------------------
  console.log("\n--- PGL1 Licenses ---");
  const licenses = (await pglProgram.account.license.all()) as any[];
  console.log(`  Total: ${licenses.length}`);
  licenses.slice(0, 10).forEach((l) => {
    console.log(
      `  ${l.publicKey.toBase58()} | holder=${fmtPubkey(l.account.holder)} | game=${fmtPubkey(l.account.game)} | issuedAt=${fmtDate(l.account.issuedAt)} | expiresAt=${l.account.expiresAt === null ? "never" : fmtDate(l.account.expiresAt)}`
    );
  });
  if (licenses.length > 10) console.log(`  ... and ${licenses.length - 10} more`);

  console.log("\n============================================================");
  console.log("  Done");
  console.log("============================================================\n");
}

main().catch((err) => {
  console.error("\n❌ Failed to fetch state:\n", err);
  process.exit(1);
});
