# Account Fetchers

Account fetchers should read typed program accounts using the deployed program ID and IDL.

Expected conventions:

- Accept a connection or provider from the caller
- Accept environment-specific program resolution
- Return typed account data
- Avoid global clients or hidden RPC configuration

Useful fetcher groups:

- PGL-1 config, game, creator state, license
- Registry config, payment token, registry game, publish grant
- Store config, payment option, purchase receipt

Fetchers should be thin wrappers over the IDL account layout and should not enforce business policy beyond decoding and basic validation.

## Fetch All Registry Games From A Frontend

The current SDK gives you the Registry IDL and the environment-aware program ID. Use Anchor in your frontend app to decode accounts.

Install app-side dependencies:

```bash
pnpm add @coral-xyz/anchor @solana/web3.js
```

Example with a wallet-adapter wallet:

```ts
import { AnchorProvider, Program, type Idl } from "@coral-xyz/anchor";
import { Connection } from "@solana/web3.js";
import {
  REGISTRY_IDL,
  getSolanaProgramId,
  type Registry
} from "@peridotvault/contracts/solana";

type RegistryGameRow = {
  publicKey: string;
  game: string;
  gameId: string;
  registeredAt: number;
  status: "active" | "suspended" | "banned" | "unknown";
};

function registryStatusToString(status: unknown): RegistryGameRow["status"] {
  if (status && typeof status === "object") {
    if ("active" in status) return "active";
    if ("suspended" in status) return "suspended";
    if ("banned" in status) return "banned";
  }

  return "unknown";
}

export async function fetchRegistryGames(wallet: AnchorProvider["wallet"]) {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed"
  });

  const registryProgramId = getSolanaProgramId({
    environment: "testnet",
    program: "registry"
  });

  const registryIdl = {
    ...REGISTRY_IDL,
    address: registryProgramId.toBase58()
  } as unknown as Idl;

  const registryProgram = new Program<Registry>(registryIdl, provider);
  const registryGames = await registryProgram.account.registryGame.all();

  return registryGames.map(({ publicKey, account }) => ({
    publicKey: publicKey.toBase58(),
    game: account.game.toBase58(),
    gameId: account.gameId,
    registeredAt: account.registeredAt.toNumber(),
    status: registryStatusToString(account.status)
  })) satisfies RegistryGameRow[];
}
```

In a React component, call this after the wallet is connected:

```ts
const games = await fetchRegistryGames(anchorWallet);
```

This only reads accounts. It does not sign, send, or create transactions.
