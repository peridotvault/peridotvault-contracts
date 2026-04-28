import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as os from "os";
import * as path from "path";

// Usage examples:
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
// # IDRX — 100,000 IDRX (9 decimals)
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set.ts registry-add EvC4a84ESadPhRFLzwD8WPqXQEk5Te8TddKDRVGqFSzh 100000000000000
// # Add IDRX to Game Store
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set.ts store-add EvC4a84ESadPhRFLzwD8WPqXQEk5Te8TddKDRVGqFSzh
// # USDT — 10 USDT (9 decimals)
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set.ts registry-add 7DFKVQY9PBpP7iMT2rwc3CKQBwuDiiZRZxaPrE5EbHCZ 10000000000
// # Add USDT to Game Store
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set.ts store-add 7DFKVQY9PBpP7iMT2rwc3CKQBwuDiiZRZxaPrE5EbHCZ

// ============================================================
//  Peridot Vault — Set Accepted Payment Tokens
// ============================================================
//  Add or update AcceptedPaymentToken for Registry & Game Store.
//
//  Usage:
//    npx ts-node scripts/set.ts registry-add  <MINT> <FEE>
//    npx ts-node scripts/set.ts registry-update <MINT> <ACTIVE> <FEE>
//    npx ts-node scripts/set.ts store-add     <MINT>
//    npx ts-node scripts/set.ts store-update  <MINT> <ACTIVE>
//
//  Examples:
//    npx ts-node scripts/set.ts registry-add EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v 1000
//    npx ts-node scripts/set.ts store-add EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
//    npx ts-node scripts/set.ts registry-update EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v true 2000
//    npx ts-node scripts/set.ts store-update EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v false
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
      `  ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set.ts ...`,
    );
  }
}

async function accountExists(connection: anchor.web3.Connection, address: PublicKey): Promise<boolean> {
  return (await connection.getAccountInfo(address)) !== null;
}

function usage() {
  console.log(`
Usage:
  npx ts-node scripts/set.ts <action> <mint> [args]

Actions:
  registry-add    <MINT> <FEE>
  registry-update <MINT> <ACTIVE(true|false)> <FEE>
  store-add       <MINT>
  store-update    <MINT> <ACTIVE(true|false)>
`);
  process.exit(1);
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length < 2) usage();

  const action = args[0];
  const mint = new PublicKey(args[1]);

  process.env.ANCHOR_PROVIDER_URL ||= "http://127.0.0.1:8899";
  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  await checkConnection(provider.connection);

  const wallet = provider.wallet as anchor.Wallet & { payer: anchor.web3.Keypair };
  const authority = wallet.payer;

  console.log("\n============================================================");
  console.log("  Peridot Vault — Set Payment Token");
  console.log("============================================================");
  console.log("  Network :", provider.connection.rpcEndpoint);
  console.log("  Wallet  :", authority.publicKey.toBase58());
  console.log("  Action  :", action);
  console.log("  Mint    :", mint.toBase58());

  const registryIdl = require("../target/idl/registry.json");
  const storeIdl = require("../target/idl/game_store.json");

  const registryProgram = new Program(registryIdl, provider) as any;
  const storeProgram = new Program(storeIdl, provider) as any;

  const registryConfigPda = derivePda([Buffer.from("registry_config")], registryProgram.programId);
  const storeConfigPda = derivePda([Buffer.from("store_config")], storeProgram.programId);

  const registryTokenPda = derivePda(
    [Buffer.from("accepted_payment_token"), mint.toBuffer()],
    registryProgram.programId,
  );
  const storeTokenPda = derivePda(
    [Buffer.from("accepted_payment_token"), mint.toBuffer()],
    storeProgram.programId,
  );

  switch (action) {
    case "registry-add": {
      if (args.length !== 3) usage();
      const fee = new anchor.BN(args[2]);
      console.log("  Fee     :", fee.toString());

      if (await accountExists(provider.connection, registryTokenPda)) {
        console.log("  ⚠️  Token already exists in registry. Use registry-update instead.");
        process.exit(0);
      }

      console.log("  Sending registry.addPaymentToken ...");
      await registryProgram.methods
        .addPaymentToken(fee)
        .accounts({
          authority: authority.publicKey,
          config: registryConfigPda,
          mint: mint,
          acceptedPaymentToken: registryTokenPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("  ✅ Registry token added");
      console.log("     PDA:", registryTokenPda.toBase58());
      break;
    }

    case "registry-update": {
      if (args.length !== 4) usage();
      const active = args[2].toLowerCase() === "true";
      const fee = new anchor.BN(args[3]);
      console.log("  Active  :", active);
      console.log("  Fee     :", fee.toString());

      if (!(await accountExists(provider.connection, registryTokenPda))) {
        console.log("  ❌ Token not found in registry. Use registry-add first.");
        process.exit(1);
      }

      console.log("  Sending registry.updatePaymentToken ...");
      await registryProgram.methods
        .updatePaymentToken(active, fee)
        .accounts({
          authority: authority.publicKey,
          config: registryConfigPda,
          mint: mint,
          acceptedPaymentToken: registryTokenPda,
        })
        .rpc();
      console.log("  ✅ Registry token updated");
      console.log("     PDA:", registryTokenPda.toBase58());
      break;
    }

    case "store-add": {
      if (args.length !== 2) usage();

      if (await accountExists(provider.connection, storeTokenPda)) {
        console.log("  ⚠️  Token already exists in store. Use store-update instead.");
        process.exit(0);
      }

      console.log("  Sending store.addPaymentToken ...");
      await storeProgram.methods
        .addPaymentToken()
        .accounts({
          authority: authority.publicKey,
          storeConfig: storeConfigPda,
          mint: mint,
          acceptedPaymentToken: storeTokenPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("  ✅ Store token added");
      console.log("     PDA:", storeTokenPda.toBase58());
      break;
    }

    case "store-update": {
      if (args.length !== 3) usage();
      const active = args[2].toLowerCase() === "true";
      console.log("  Active  :", active);

      if (!(await accountExists(provider.connection, storeTokenPda))) {
        console.log("  ❌ Token not found in store. Use store-add first.");
        process.exit(1);
      }

      console.log("  Sending store.updatePaymentToken ...");
      await storeProgram.methods
        .updatePaymentToken(active)
        .accounts({
          authority: authority.publicKey,
          storeConfig: storeConfigPda,
          acceptedPaymentToken: storeTokenPda,
        })
        .rpc();
      console.log("  ✅ Store token updated");
      console.log("     PDA:", storeTokenPda.toBase58());
      break;
    }

    default:
      usage();
  }

  console.log("============================================================\n");
}

main().catch((err) => {
  console.error("\n❌ Failed to set payment token:\n", err);
  process.exit(1);
});
