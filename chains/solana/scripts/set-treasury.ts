import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as os from "os";
import * as path from "path";

// ============================================================
//  Peridot Vault — Set Treasury across all programs
// ============================================================
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set-treasury.ts
//  Updates treasury in PGL1, Registry, and Game Store.
//
//  Usage:
//    npx ts-node scripts/set-treasury.ts
//
//  Optional env:
//    PERIDOT_TREASURY=<pubkey>   (default below)
//    ANCHOR_PROVIDER_URL         (default: http://127.0.0.1:8899)
// ============================================================

const DEFAULT_TREASURY = "EjXj948Fe5YGFLzRPDgkaiLoqs4MAzA6M8zrPv4peKoH";

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
      `  ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set-treasury.ts`,
    );
  }
}

async function accountExists(connection: anchor.web3.Connection, address: PublicKey): Promise<boolean> {
  return (await connection.getAccountInfo(address)) !== null;
}

async function main() {
  process.env.ANCHOR_PROVIDER_URL ||= "http://127.0.0.1:8899";
  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const newTreasury = new PublicKey(process.env.PERIDOT_TREASURY || DEFAULT_TREASURY);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  await checkConnection(provider.connection);

  const wallet = provider.wallet as anchor.Wallet & { payer: anchor.web3.Keypair };
  const authority = wallet.payer;

  console.log("\n============================================================");
  console.log("  Peridot Vault — Set Treasury");
  console.log("============================================================");
  console.log("  Network    :", provider.connection.rpcEndpoint);
  console.log("  Authority  :", authority.publicKey.toBase58());
  console.log("  New Treasury:", newTreasury.toBase58());

  const pglIdl = require("../target/idl/pgl1.json");
  const registryIdl = require("../target/idl/registry.json");
  const storeIdl = require("../target/idl/game_store.json");

  const pglProgram = new Program(pglIdl, provider) as any;
  const registryProgram = new Program(registryIdl, provider) as any;
  const storeProgram = new Program(storeIdl, provider) as any;

  const pglConfigPda = derivePda([Buffer.from("pgl_config")], pglProgram.programId);
  const registryConfigPda = derivePda([Buffer.from("registry_config")], registryProgram.programId);
  const storeConfigPda = derivePda([Buffer.from("store_config")], storeProgram.programId);

  // --- PGL1 ---
  console.log("\n--- PGL1 ---");
  if (await accountExists(provider.connection, pglConfigPda)) {
    const before = (await pglProgram.account.pglConfig.fetch(pglConfigPda)) as any;
    if (before.treasury.toBase58() === newTreasury.toBase58()) {
      console.log("  Treasury already set correctly. Skipping.");
    } else {
      console.log("  Updating treasury ...");
      await pglProgram.methods
        .setTreasury(newTreasury)
        .accounts({
          authority: authority.publicKey,
          pglConfig: pglConfigPda,
        })
        .rpc();
      console.log("  ✅ PGL1 treasury updated");
    }
  } else {
    console.log("  ⚠️ PglConfig not found. Run config.ts first.");
  }

  // --- Registry ---
  console.log("\n--- Registry ---");
  if (await accountExists(provider.connection, registryConfigPda)) {
    const before = (await registryProgram.account.registryConfig.fetch(registryConfigPda)) as any;
    if (before.treasury.toBase58() === newTreasury.toBase58()) {
      console.log("  Treasury already set correctly. Skipping.");
    } else {
      console.log("  Updating treasury ...");
      await registryProgram.methods
        .setTreasury(newTreasury)
        .accounts({
          authority: authority.publicKey,
          config: registryConfigPda,
        })
        .rpc();
      console.log("  ✅ Registry treasury updated");
    }
  } else {
    console.log("  ⚠️ RegistryConfig not found. Run config.ts first.");
  }

  // --- Game Store ---
  console.log("\n--- Game Store ---");
  if (await accountExists(provider.connection, storeConfigPda)) {
    const before = (await storeProgram.account.storeConfig.fetch(storeConfigPda)) as any;
    if (before.treasury.toBase58() === newTreasury.toBase58()) {
      console.log("  Treasury already set correctly. Skipping.");
    } else {
      console.log("  Updating treasury ...");
      await storeProgram.methods
        .setTreasury(newTreasury)
        .accounts({
          authority: authority.publicKey,
          storeConfig: storeConfigPda,
        })
        .rpc();
      console.log("  ✅ Game Store treasury updated");
    }
  } else {
    console.log("  ⚠️ StoreConfig not found. Run config.ts first.");
  }

  console.log("\n============================================================");
  console.log("  Done");
  console.log("============================================================\n");
}

main().catch((err) => {
  console.error("\n❌ Failed to set treasury:\n", err);
  process.exit(1);
});
