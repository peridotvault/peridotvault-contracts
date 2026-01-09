import { network } from "hardhat";
import { keccak256, toBytes, parseEther } from "viem";

async function main() {
    const { viem } = await network.connect();
    const publicClient = await viem.getPublicClient();
    const [deployer, buyer] = await viem.getWalletClients();

    // 1) Deploy (sama seperti deploy-pgc1.ts)
    const tokenURI1155 = "ipfs://base/{id}.json";
    const initialContractMetaURI = "ipfs://contract/v1.json";
    const initialContractMetaHash = keccak256(toBytes(initialContractMetaURI));

    const gameId = keccak256(toBytes("peridot:studio:my-game"));
    const paymentToken = "0x0000000000000000000000000000000000000000";

    const initialPrice = parseEther("0.01");
    const initialMaxSupply = 0n;
    const treasuryRouter = deployer.account.address;
    const developerRecipient = deployer.account.address;
    const platformFeeBps = 1000;

    const pgc1 = await viem.deployContract("PGC1", [
        tokenURI1155,
        initialContractMetaHash,
        initialContractMetaURI,
        gameId,
        paymentToken,
        initialPrice,
        initialMaxSupply,
        treasuryRouter,
        developerRecipient,
        platformFeeBps,
    ]);

    console.log("PGC1:", pgc1.address);

    // 2) Publish game metadata (required before buy)
    const metaUriV1 = "ipfs://game/meta-v1.json";
    const metaHashV1 = keccak256(toBytes(metaUriV1)); // dev-only; idealnya hash dari bytes JSON metadata

    const publishTx = await pgc1.write.publishMetadata([metaHashV1, metaUriV1]);
    await publicClient.waitForTransactionReceipt({ hash: publishTx });

    console.log("metadataHeadVersion:", (await pgc1.read.metadataHeadVersion()).toString());
    console.log("metadataHeadHash:", await pgc1.read.metadataHeadHash());

    // 3) Buy from buyer account
    const pgc1Buyer = await viem.getContractAt("PGC1", pgc1.address, {
        client: { wallet: buyer },
    });

    const buyTx = await pgc1Buyer.write.buy([], { value: initialPrice });
    await publicClient.waitForTransactionReceipt({ hash: buyTx });

    // 4) Check balance (ERC1155 balanceOf takes [account, id])
    const licenseId = 1n;
    const bal = await pgc1.read.balanceOf([buyer.account.address, licenseId]);

    console.log("Buyer:", buyer.account.address);
    console.log("Buyer license balance:", bal.toString());

    // 5) Show last MetadataPublished + Purchased events (optional)
    const fromBlock = 0n;
    const metaLogs = await publicClient.getContractEvents({
        address: pgc1.address,
        abi: pgc1.abi,
        eventName: "MetadataPublished",
        fromBlock,
        strict: true,
    });

    const purchaseLogs = await publicClient.getContractEvents({
        address: pgc1.address,
        abi: pgc1.abi,
        eventName: "Purchased",
        fromBlock,
        strict: true,
    });

    console.log("MetadataPublished events:", metaLogs.length);
    console.log("Purchased events:", purchaseLogs.length);

    if (metaLogs.length > 0) {
        const last = metaLogs[metaLogs.length - 1]!;
        console.log("Last metadata uri:", last.args.uri);
        console.log("Last metadata version:", last.args.version.toString());
    }
}

main().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});
