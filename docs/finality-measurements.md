# PGC1 M2 finality-delay measurement (ANT-487, testnet-only)

Scope: localnet measured values + testnet methodology. **No mainnet txs.**
No mainnet RPCs were touched; `scripts/measure-finality.sh` refuses
mainnet-looking endpoints (fail-closed substring guard).

## 1. Measured: localnet (2026-10-24)

### EVM — Hardhat in-process chain (chainId 31337)
- Automine: txs confirm in the next block, `latest == finalized` on the
  ephemeral dev chain (no reorg source exists). The full M2 suite
  (`chains/evm/test/M2Attestation.ts`, 12 tests) + M1 suite (16 tests) run
  green with zero finality wait.
- `XChainAttester.consume` (3-of-5): **106,447 gas** (NFR bound 400k).

### Solana — solana-test-validator 2.3.13, single node
- `consume_xchain_attestation` (paid, wormhole label, 3-of-5) landed in
  **slot 654**, `slots_elapsed ~1` (processed → finalized), wall
  **time_to_finalized = 13,066 ms** on an idle single-node validator.
  The 13 s is an idle-validator artifact (vote rooting on a quiet node),
  not a protocol floor — devnet confirmation is normally faster; the
  testnet row below must be measured, not inferred.
- Compute: **335,879 CU consumed** of a 600,000 budget. The default 200,000
  CU limit is INSUFFICIENT (tx fails `exceeded CUs`) — every testnet/mainnet
  consume MUST prepend `ComputeBudgetProgram.setComputeUnitLimit` (≥400k).
  This is a runbook requirement, not an optimization.
- Negatives proven on localnet: replay (`Allocate ... already in use` at
  PDA init), tampered paidAmount (`UnknownAttester`), future `observedAt`
  (`AttestationObservedAtInFuture`).

## 2. Testnet methodology (to run before any testnet attestation)

Run (RPC URLs supplied at runtime, never committed):
```bash
RPC_URL=<base-sepolia-rpc>  ./scripts/measure-finality.sh 10 eip155
RPC_URL=<lisk-sepolia-rpc>  ./scripts/measure-finality.sh 10 eip155
RPC_URL=<solana-devnet-rpc> ./scripts/measure-finality.sh 10 solana
```
Record `latest − finalized` lag over ≥10 samples × 3 runs (quiet/busy).
The attester signs ONLY home-finalized observations
(`homeFinalizedBlock` + `observedAt` bound in the digest); the dest
contract/program enforces `reorgDelay` AFTER `observedAt`.

### Initial reorgDelay policy (CONFIRM via script before testnet use)

| Home chain | Initial reorgDelay | Rationale (to confirm) |
|---|---|---|
| eip155:84532 (Base Sepolia) | 3600 s | OP-Stack derivation finalizes via L1 Sepolia; 1 h comfortably exceeds derivation windows |
| eip155:4202 (Lisk Sepolia) | 3600 s | Same OP-Stack lineage as Base Sepolia |
| solana localnet/devnet | 60 s | Devnet `finalized` trails optimistic confirmation by ~32 slots (~13–30 s); 60 s doubles it |

Localnet fixtures already encode the short-but-enforced pattern
(`registry.localnet.json` timelock 60 s; short ≠ 0).

## 3. Deployed-program-vs-source equivalence (for future value-bearing use)

- Committed-source build: `cargo build-sbf -- --package registry` at this
  branch → `registry.so` **sha256
  `8bded127489463727a70ab54c8db857605a98e387010a7460e9b8723749d605a`**,
  `declare_id G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE` embedded
  (verified present exactly once; no temp ids).
- Localnet run used a TEMP-ID build (`H1yS7Rmi…`, sha256
  `4ff60a49860aedd59d9b6bb03c1f8dba33bd4fc44e0e6977619b6f05b412`)
  because Anchor enforces declared-id == deployed-id; the temp build was
  never committed and the source was reverted + rebuilt deterministically
  (same sha256 as the pre-temp build).
- Rule: any value-bearing deploy MUST be a reproducible build from the
  merged `paperclip` source (e.g. `solana-verify`), with the digest
  recorded in the deploy ticket — and NEVER executed by an agent without
  an explicit board order (mainnet/value deploys are board-only).

## 4. Residual risks → Tobias (security review)
- Signer rotation does not invalidate prior digests (no epoch in the
  envelope) — operational note, rotation is owner-governed until ANT-486.
- EVM `consume` accepts >3 signatures (breaks at quorum); extra sigs cost
  the caller gas only — no griefing vector beyond self-pay.
- Solana single-global `reorg_delay` (per-home-chain map is a small
  extension, same shape as the EVM mapping).
