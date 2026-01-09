import { network } from "hardhat";
import { keccak256, toBytes, parseEther } from "viem";

async function main() {
    const { viem } = await network.connect();
    const publicClient = await viem.getPublicClient();
    const [deployer] = await viem.getWalletClients();

    const tokenURI1155 = "ipfs://base/{id}.json";

    const initialContractMetaURI = "ipfs://contract/v1.json";
    const initialContractMetaHash = keccak256(toBytes(initialContractMetaURI));

    const gameId = keccak256(toBytes("peridot:studio:my-game"));

    const paymentToken = "0x0000000000000000000000000000000000000000"; // ETH
    const initialPrice = parseEther("0.01");
    const initialMaxSupply = 0n;

    const treasuryRouter = deployer.account.address;
    const developerRecipient = deployer.account.address;
    const platformFeeBps = 1000;

    console.log("Deployer:", deployer.account.address);
    console.log("ChainId:", await publicClient.getChainId());

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

    console.log("PGC1 deployed at:", pgc1.address);

    // sanity reads
    console.log("price:", (await pgc1.read.price()).toString());
    console.log("maxSupply:", (await pgc1.read.maxSupply()).toString());
    console.log("contractMetaHeadVersion:", (await pgc1.read.contractMetaHeadVersion()).toString());
}

main().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});
