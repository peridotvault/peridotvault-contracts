// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// @title XChainEnvelope — PGC1 M1 envelope payment binding (ANT-490)
/// @notice Implements parent ANT-485 `spec` §3.4 (Tobias [ANT-479] residual
///         risk #1): the attester MUST bind is_free/paid_amount into the
///         EIP-712 `PGL1-XCHAIN` envelope. Dest consumption rules are
///         normative here; the full round-trip tests land in [ANT-487].
///         Lens: never infer free-from-zero (both sides of the M0 H3
///         invariant are re-asserted cross-chain).
library XChainEnvelope {
    /// Envelope/payment schema version (M0 M5 lineage, CURRENT_VERSION = 1).
    uint8 constant CURRENT_VERSION = 1;
    uint8 constant TOKEN_FORMAT_EVM20 = 0;
    uint8 constant TOKEN_FORMAT_SOLANA32 = 1;

    struct Envelope {
        bool isFree;
        uint256 paidAmount;
        bytes paidToken;
        uint8 paidTokenFormat;
        uint8 receiptVersion;
        uint8 version;
    }

    error VersionMismatch();
    error UnsupportedEnvelopeVersion();
    error PaidAmountZeroOnPaidPath();
    error PaidAmountNonZeroOnFreePath();
    error PaidTokenNonEmptyOnFreePath();
    error InvalidTokenFormat();

    /// Destination verification (AC-ENV-1..3). Pure so dest chains can call
    /// it inside their consume path; reverts carry the SPECIFIC reason.
    function verifyForDest(Envelope memory e) internal pure {
        if (e.version != e.receiptVersion) revert VersionMismatch();
        if (e.version > CURRENT_VERSION) revert UnsupportedEnvelopeVersion();
        if (!e.isFree && e.paidAmount == 0)
            revert PaidAmountZeroOnPaidPath();
        if (e.isFree && e.paidAmount != 0)
            revert PaidAmountNonZeroOnFreePath();
        if (e.isFree && e.paidToken.length != 0)
            revert PaidTokenNonEmptyOnFreePath();
        if (
            e.paidTokenFormat != TOKEN_FORMAT_EVM20 &&
            e.paidTokenFormat != TOKEN_FORMAT_SOLANA32
        ) revert InvalidTokenFormat();
    }
}
