# M1 fixtures (ANT-490, parent ANT-485 spec §3.5)

- `gameid.vectors.json` — 5 cross-stack gameId vectors. Vectors 1–3 reproduce
  the spec §3.1 worked examples byte-identically. Vectors 4–5 carry a
  `_deviation` note: as printed, the spec vectors do not reproduce under the
  normative `keccak256(abi.encode(...))` derivation (vector 5's publisher hex
  is 33 bytes; vector 4's chainId is ellipsis-truncated), and the §3.1
  "constant 0x60/0xa0/0xe0 offsets" table is only valid for short inputs —
  head offsets grow with preceding tail lengths. The formula governs, not the
  table. Both stacks implement dynamic offsets; EVM `AC-GAMEID-2` and the
  Solana `gameid_vectors_cross_stack` unit test prove equivalence over the
  corrected vectors. Flagged to Theo (spec owner) for a spec amendment.
- `registry.localnet.json` — localnet entries (`eip155:31337`,
  `solana:localnet`), `timelockDelay: 60` (short but enforced — short ≠ absent).
  `programId: null` = not yet deployed, never a live address.
- `registry.testnets.json` — Base/Lisk Sepolia + Solana devnet entries with
  `"fixtureOnly": true` PLACEHOLDER ids. Loaders must refuse to treat them as
  live (fail-closed read). Q3 (Tristan) ruling pending on placeholder-vs-omitted.

Gate: `scripts/check-no-mainnet.sh` (AC-FIX-1) — zero mainnet IDs, zero secrets.
