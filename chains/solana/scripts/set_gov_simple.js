const { Connection, PublicKey, Keypair, Transaction, TransactionInstruction } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");

// Load wallet
const idPath = path.join(process.env.HOME, ".config/solana/id.json");
const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(idPath, 'utf8')));
const wallet = Keypair.fromSecretKey(secretKey);

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const REGISTRY_PID = new PublicKey("3bUSqLjWxUgmruzuRwhtWwhV93b4RXVN7bE5qHxHHxLj");
const STORE_PID = new PublicKey("DSiyompbYR2k2GsS69FWkvE9N3vf32Da4JNqZKYvn2Pp");
const FACTORY_PID = new PublicKey("3EaXmAr9wAvYgXhz1BH4Kpa5DDCc5oTykeeGtBHeqYXA");

const NEW_GOV = new PublicKey("EjXj948Fe5YGFLzRPDgkaiLoqs4MAzA6M8zrPv4peKoH");

// Discriminators
const SET_GOV_DISC = Buffer.from([34, 71, 128, 245, 179, 42, 140, 137]);
const SET_TREASURY_DISC = Buffer.from([57, 97, 196, 95, 195, 206, 106, 136]);

async function runSet(label, pid, pdaSeed, disc, newKey) {
    console.log(`\nUpdating ${label} (${disc === SET_GOV_DISC ? "Gov" : "Treasury"})...`);
    
    const [pda] = PublicKey.findProgramAddressSync([Buffer.from(pdaSeed)], pid);
    
    const data = Buffer.concat([disc, newKey.toBuffer()]);
    
    const ix = new TransactionInstruction({
        keys: [
            { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
            { pubkey: pda, isSigner: false, isWritable: true },
        ],
        programId: pid,
        data: data
    });
    
    const tx = new Transaction().add(ix);
    try {
        const sig = await connection.sendTransaction(tx, [wallet]);
        await connection.confirmTransaction(sig);
        console.log(`✅ ${label} Success: ${sig}`);
    } catch (e) {
        console.log(`❌ ${label} Failed: ${e.message}`);
    }
}

async function main() {
    console.log("Wallet:", wallet.publicKey.toBase58());
    
    // Registry
    await runSet("Registry", REGISTRY_PID, "registry_state", SET_GOV_DISC, NEW_GOV);
    await runSet("Registry", REGISTRY_PID, "registry_state", SET_TREASURY_DISC, NEW_GOV);
    
    // Store
    await runSet("Store", STORE_PID, "game_store_state", SET_GOV_DISC, NEW_GOV);
    await runSet("Store", STORE_PID, "game_store_state", SET_TREASURY_DISC, NEW_GOV);
    
    // Factory (using EjXj as target, but EjXj is already current gov based on get.ts output?)
    // If EjXj is already current gov, WE cannot sign for it.
    await runSet("Factory", FACTORY_PID, "factory_state", SET_GOV_DISC, NEW_GOV);
}

main();
