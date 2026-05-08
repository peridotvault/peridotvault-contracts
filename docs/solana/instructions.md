# Instruction Builders

Instruction builders should create Solana `TransactionInstruction` values for a single program instruction.

Expected conventions:

- Accept explicit accounts from the caller
- Derive common PDAs when safe and deterministic
- Accept an environment or program ID override
- Never sign transactions
- Never send transactions

Instruction builders are useful when an app already owns transaction assembly and wants precise control over ordering, compute budget instructions, lookup tables, and priority fees.

Current SDK work is focused on stable IDLs, deployment metadata, and PDA helpers. Builder APIs should be added only after the account contracts are finalized.
