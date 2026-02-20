import { network } from "hardhat";
import { formatEther, parseEther, type Address } from "viem";
import fs from "node:fs";
import path from "node:path";

type Deployments = {
    networkName: string;
    chainId: number;
    confirmations: number;

    deployer: Address;

    pgc1Implementation: Address;
    registry: Address;
    factory: Address;

    treasuryRouter: Address;
    feeToken: Address;
    publishFeeWei: string;
    platformFeeBps: number;

    deployedAt: string;
};

const ZERO_ADDRESS =
    "0x0000000000000000000000000000000000000000" as const satisfies Address;

// ==========================
// CONFIG (adjust if needed)
// ==========================

// default treasury (can be overridden via ENV)
const TREASURY_ADDRESS = (process.env.TREASURY_ADDRESS ??
    "0xe55a693527d8CD166a9b814BfFdAA5Adb65DB5aB") as Address;

// publish fee config
const FEE_TOKEN = ZERO_ADDRESS; // ETH
const PUBLISH_FEE = parseEther("0.000001");

// IMPORTANT: platform fee (bps)
// 0     = 0%   (beta)
// 500   = 5%
// 1000  = 10%
const PLATFORM_FEE_BPS = 500;

// ==========================

function nowIso() {
    return new Date().toISOString();
}

function ensureDir(dir: string) {
    fs.mkdirSync(dir, { recursive: true });
}

function writeJsonAtomic(filePath: string, data: unknown) {
    const tmp = `${filePath}.tmp`;
    fs.writeFileSync(tmp, JSON.stringify(data, null, 2));
    fs.renameSync(tmp, filePath);
}

function getConfirmations(networkName: string): number {
    if (networkName === "hardhat" || networkName === "localhost") return 1;
    return 1;
}

async function waitTx(
    client: {
        waitForTransactionReceipt: (args: {
            hash: `0x${string}`;
            confirmations: number;
        }) => Promise<{ status: string }>;
    },
    hash: `0x${string}`,
    confirmations: number,
    label: string
) {
    const receipt = await client.waitForTransactionReceipt({
        hash,
        confirmations,
    });
    if (receipt.status !== "success") {
        throw new Error(`${label} reverted: ${hash}`);
    }
}

async function main() {
    const { viem, networkName } = await network.connect();
    const client = await viem.getPublicClient();
    const [walletClient] = await viem.getWalletClients();

    const deployer = walletClient.account.address as Address;
    const chainId = await client.getChainId();
    const confirmations = getConfirmations(networkName);

    console.log(`\nDeploying Peridot contracts to ${networkName}...`);
    console.log("Deployer:", deployer);
    console.log("ChainId:", chainId);

    const balance = await client.getBalance({ address: deployer });
    console.log("Balance:", formatEther(balance));

    // =============================================================
    // 1) Deploy PGC1 implementation
    // =============================================================
    console.log("\n[1/5] Deploying PGC1 implementation...");
    const pgc1Impl = await viem.deployContract("PGC1", []);
    console.log("PGC1_IMPL:", pgc1Impl.address);

    // =============================================================
    // 2) Deploy Registry
    // =============================================================
    console.log("\n[2/5] Deploying PeridotRegistry...");
    const registry = await viem.deployContract("PeridotRegistry", []);
    console.log("REGISTRY:", registry.address);

    // =============================================================
    // 3) Deploy Factory
    // =============================================================
    console.log("\n[3/5] Deploying PGC1Factory...");
    const factory = await viem.deployContract("PGC1Factory", [
        pgc1Impl.address,
        TREASURY_ADDRESS,
        FEE_TOKEN,
        PUBLISH_FEE,
    ]);
    console.log("FACTORY:", factory.address);

    // =============================================================
    // 4) Wire registry <-> factory (idempotent)
    // =============================================================
    console.log("\n[4/5] Wiring registry <-> factory...");

    const currentFactory = (await registry.read.factory()) as Address;
    if (currentFactory.toLowerCase() !== factory.address.toLowerCase()) {
        const tx = await registry.write.setFactory([factory.address]);
        console.log("Registry.setFactory tx:", tx);
        await waitTx(client, tx, confirmations, "Registry.setFactory");
        console.log("Registry.setFactory OK");
    } else {
        console.log("Registry.setFactory skipped (already set)");
    }

    const currentRegistry = (await factory.read.registry()) as Address;
    if (currentRegistry.toLowerCase() !== registry.address.toLowerCase()) {
        const tx = await factory.write.setRegistry([registry.address]);
        console.log("Factory.setRegistry tx:", tx);
        await waitTx(client, tx, confirmations, "Factory.setRegistry");
        console.log("Factory.setRegistry OK");
    } else {
        console.log("Factory.setRegistry skipped (already set)");
    }

    // =============================================================
    // 5) Set platform fee BPS (IMPORTANT)
    // =============================================================
    console.log("\n[5/5] Setting platform fee BPS...");

    const currentBps = Number(await factory.read.platformFeeBps());
    if (currentBps !== PLATFORM_FEE_BPS) {
        const tx = await factory.write.setPlatformFeeBps([PLATFORM_FEE_BPS]);
        console.log("Factory.setPlatformFeeBps tx:", tx);
        await waitTx(client, tx, confirmations, "Factory.setPlatformFeeBps");
        console.log("Factory.setPlatformFeeBps OK");
    } else {
        console.log("PlatformFeeBps skipped (already set)");
    }

    // =============================================================
    // Save deployments
    // =============================================================
    const out: Deployments = {
        networkName,
        chainId,
        confirmations,

        deployer,

        pgc1Implementation: pgc1Impl.address,
        registry: registry.address,
        factory: factory.address,

        treasuryRouter: TREASURY_ADDRESS,
        feeToken: FEE_TOKEN,
        publishFeeWei: PUBLISH_FEE.toString(),
        platformFeeBps: PLATFORM_FEE_BPS,

        deployedAt: nowIso(),
    };

    const outDir = path.join(process.cwd(), "deployments", networkName);
    ensureDir(outDir);

    const outPath = path.join(outDir, `${chainId}.json`);
    writeJsonAtomic(outPath, out);

    console.log("\nSaved deployments:", outPath);
    console.log("\nDeployment successful!\n");
}

main().catch((err) => {
    console.error("\nDEPLOY FAILED:");
    console.error(err);
    process.exitCode = 1;
});
