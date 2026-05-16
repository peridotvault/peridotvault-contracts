# PDA Reference

The SDK exposes PDA helpers so integrators do not manually duplicate seed logic.

```ts
import { findLicensePda } from "@peridotvault/contracts/solana";

const [licensePda, bump] = findLicensePda(holder, game);
```

## PGL-1

- `findPglConfigPda`
- `findAuthorizedActorPda`
- `findCreatorStatePda`
- `findGamePda`
- `findLicensePda`

## Registry

- `findRegistryConfigPda`
- `findRegistryPaymentTokenPda`
- `findRegistryGamePda`
- `findPublishGrantPda`

## Game Store

- `findStoreConfigPda`
- `findAuthorizedSourceProgramPda`
- `findAuthorizedRegistryProgramPda`
- `findStoreAcceptedPaymentTokenPda`
- `findGameStoreConfigPda`
- `findGamePaymentOptionPda`
- `findPurchaseReceiptPda`

All PDA helpers use `PublicKey.findProgramAddressSync`.
