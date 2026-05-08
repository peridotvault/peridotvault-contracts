# Cross-Program CPI Audit

Cross-program invocation is a primary security boundary for PeridotVault.

Review areas:

- Registry to PGL-1 game creation
- Registry to Game Store configuration
- Game Store to PGL-1 license minting
- Program ID validation
- PDA checks across program boundaries
- Account ownership validation

Audit questions:

- Can a caller substitute a malicious program?
- Are CPI accounts constrained by both seed and owner?
- Are signer seeds scoped to the intended PDA only?
- Are authority assumptions documented and tested?

Status: initial report placeholder.
