import { network } from "hardhat";
import { keccak256, toBytes, parseEther } from "viem";
import fs from "node:fs";

type Deployments = {
    factory: `0x${string}`;
    registry: `0x${string}`;
    publishFee: string; // BigInt string
};

function loadDeployments(): Deployments {
    const p = "deployments/localhost.json";
    if (!fs.existsSync(p)) throw new Error(`Missing ${p}. Run run_test.sh first.`);
    return JSON.parse(fs.readFileSync(p, "utf8")) as Deployments;
}

async function main() {
    const { viem } = await network.connect();
    const publicClient = await viem.getPublicClient();
    const [publisher] = await viem.getWalletClients();

    const dep = loadDeployments();
    const publishFee = BigInt(dep.publishFee);

    const factory = await viem.getContractAt("PGC1Factory", dep.factory);
    const registry = await viem.getContractAt("PeridotRegistry", dep.registry);

    console.log("Publisher:", publisher.account.address);
    console.log("Factory:", factory.address);
    console.log("Registry:", registry.address);
    console.log("Publish fee:", publishFee.toString());

    // Shared placeholders (you can replace later with IPFS / your CDN)
    const tokenURI1155 = "https://metadata.peridotvault.dev/pgc1/{id}.json";

    // publish 5 games
    for (let i = 1; i <= 5; i++) {
        const slug = `peridot:studio:game-${i}`;
        const gameId = keccak256(toBytes(slug));

        // Contract-level metadata for this game contract (commit v1)
        const contractMetaURI = `https://metadata.peridotvault.dev/contracts/game-${i}/contract-v1.json`;
        const contractMetaHash = keccak256(toBytes(contractMetaURI)); // dev placeholder hash

        // Example sale config
        const paymentToken = "0x0000000000000000000000000000000000000000" as const; // ETH
        const price = parseEther("0.01"); // 0.01 ETH
        const maxSupply = 0n; // unlimited

        const init = {
            tokenURI1155,
            initialContractMetaHash: contractMetaHash,
            initialContractMetaURI: contractMetaURI,
            gameId,
            paymentToken,
            price,
            maxSupply,
            treasuryRouter: publisher.account.address,
            developerRecipient: publisher.account.address,
            platformFeeBps: 1000,
        } as const;

        console.log(`\n[${i}/5] Publishing game: ${slug}`);
        const txHash = await factory.write.publishGame([init], { value: publishFee });
        const receipt = await publicClient.waitForTransactionReceipt({ hash: txHash });

        console.log("tx:", txHash);
        console.log("block:", receipt.blockNumber.toString(), "status:", receipt.status);

        // Read GamePublished event in that block to get pgc1 address
        const logs = await publicClient.getContractEvents({
            address: factory.address,
            abi: factory.abi,
            eventName: "GamePublished",
            fromBlock: receipt.blockNumber,
            toBlock: receipt.blockNumber,
            strict: true,
        });

        const ourLog = logs.find((l) => l.args.gameId === gameId && l.args.publisher === publisher.account.address);
        if (!ourLog) {
            console.log("WARNING: Could not find GamePublished log in receipt block.");
            continue;
        }

        const pgc1 = ourLog.args.pgc1;
        console.log("PGC1:", pgc1);

        // Verify registry entry exists
        const reg = await registry.read.games([gameId]);
        // reg = [pgc1, publisher, createdAt, active] (based on your ABI)
        const regPgc1 = reg[0];
        const regPublisher = reg[1];

        console.log("Registry.pgc1:", regPgc1);
        console.log("Registry.publisher:", regPublisher);

        if (regPgc1.toLowerCase() !== pgc1.toLowerCase()) {
            console.log("WARNING: Registry pgc1 mismatch!");
        }
        if (regPublisher.toLowerCase() !== publisher.account.address.toLowerCase()) {
            console.log("WARNING: Registry publisher mismatch!");
        }
    }

    console.log("\nDone. 5 games published.");
}

main().catch((e) => {
    console.error(e);
    process.exitCode = 1;
});
