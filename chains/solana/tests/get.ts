import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey, Connection } from "@solana/web3.js";
import { expect } from "chai";

import { Registry } from "../target/types/registry";
import { GameStore } from "../target/types/game_store";
import { Pgc1 } from "../target/types/pgc1";

describe("PeridotVault - GET TEST (LOCALNET, READ ONLY)", () => {
    // ==============================
    // 🔥 CONNECTION (NO WALLET)
    // ==============================

    const connection = new Connection(
        "http://127.0.0.1:8899",
        "confirmed"
    );

    const dummyWallet = {
        publicKey: PublicKey.default,
        signTransaction: async (tx: any) => tx,
        signAllTransactions: async (txs: any) => txs,
    };

    const provider = new AnchorProvider(connection, dummyWallet as any, {});
    // Removed anchor.setProvider(provider) as it breaks other tests.

    // ==============================
    // PROGRAMS
    // ==============================

    const registryProgram = anchor.workspace.Registry as Program<Registry>;
    const pgcProgram = anchor.workspace.Pgc1 as Program<Pgc1>;
    const storeProgram = anchor.workspace.GameStore as Program<GameStore>;

    // ==============================
    // PDA
    // ==============================

    const [registryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("registry_state")],
        registryProgram.programId
    );

    const [storePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("game_store_state")],
        storeProgram.programId
    );

    const [globalPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("global_state")],
        pgcProgram.programId
    );

    // ==============================
    // 🔧 SAFE FETCH (ANTI CRASH)
    // ==============================

    async function safeFetch(label: string, fn: () => Promise<any>) {
        try {
            const res = await fn();
            console.log(`✅ ${label} EXISTS`);
            return res;
        } catch (e: any) {
            console.log(`❌ ${label} NOT INITIALIZED`);
            return null;
        }
    }

    // ==============================
    // TEST
    // ==============================

    it("🔥 FULL STATE CHECK", async () => {
        console.log("\n==============================");
        console.log("🔍 CHECKING CONTRACT STATE...");
        console.log("==============================\n");

        // ==============================
        // REGISTRY
        // ==============================

        const registry = await safeFetch("REGISTRY", () =>
            registryProgram.account.registryConfig.fetch(registryPda)
        );

        if (registry) {
            console.log("\n===== REGISTRY =====");
            console.log("PDA:", registryPda.toBase58());
            console.log("Governance:", registry.governance.toBase58());
            console.log("Treasury:", registry.treasury.toBase58());

            const registrations = await registryProgram.account.registryGameAccount.all();
            console.log("Total Games:", registrations.length);
        }

        // ==============================
        // PGC1
        // ==============================

        // PGC1 Global state check removed as it no longer exists.

        // ==============================
        // STORE
        // ==============================

        const store = await safeFetch("STORE", () =>
            storeProgram.account.storeConfig.fetch(storePda)
        );

        if (store) {
            console.log("\n===== STORE =====");
            console.log("PDA:", storePda.toBase58());
            console.log("Governance:", store.governance.toBase58());
            console.log("Treasury:", store.treasury.toBase58());
            console.log("Registry:", store.registry.toBase58());
            console.log("Platform Fee Bps:", store.platformFeeBps);
        }

        // ==============================
        // CROSS VALIDATION
        // ==============================

        if (registry && store) {
            console.log("\n===== CONSISTENCY CHECK =====");
            expect(store.registry.toBase58()).to.equal(registryPda.toBase58());
            console.log("✅ All contracts are linked correctly");
        }

        // ==============================
        // GAMES
        // ==============================

        const registrations = await registryProgram.account.registryGameAccount.all();
        console.log("\n===== GAMES =====");

        if (registrations.length === 0) {
            console.log("⚠️ No games found");
        }

        for (const reg of registrations) {
            const game = reg.account;
            console.log("----------------------");
            console.log("Game ID:", game.gameId);
            console.log("Registry PDA:", reg.publicKey.toBase58());
            console.log("Status:", game.active ? "ACTIVE" : "INACTIVE");
        }

        console.log("\n==============================");
        console.log("✅ DONE");
        console.log("==============================\n");
    });
});
