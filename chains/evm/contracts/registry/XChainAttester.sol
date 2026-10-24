// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {EIP712} from "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

import {XChainEnvelope} from "./XChainEnvelope.sol";

/// @title XChainAttester — PGC1 M2 3-of-5 attestation verifier (ANT-487)
/// @notice Home chain emits a license event; an off-chain 3-of-5 ECDSA
///         (secp256k1) attester set signs the EIP-712 `PGL1-XCHAIN` envelope;
///         this contract verifies the quorum and records dest-side
///         RECOGNITION (no mint — recognition-only in v1, no Solana native
///         mint per scope). Transport-agnostic: `transportId` is an opaque
///         label bound into the digest (e.g. keccak("wormhole") or
///         keccak("layerzero-v2")) — the verifier never interprets it, so
///         Wormhole-primary / LZ-V2-fallback are swappable at low cost with
///         no transport lock-in (handoff §9).
///
///         Payment binding (Tobias [ANT-479] residual risk #1): `isFree` /
///         `paidAmount` (+ `paidToken`, `receiptVersion`) are struct fields,
///         hence signed by the quorum AND re-checked dest-side via
///         `XChainEnvelope.verifyForDest` — never infer free-from-zero.
///
///         Lenses:
///         - Gas: calldata args; cheap checks (version, payment rules, sig
///           count) short-circuit before the ecrecover loop; `licensee` is a
///           single bytes32 (EVM addr left-padded / Solana pubkey raw).
///         - Reentrancy & CEI: `consume` makes ZERO external calls — effects
///           (consumed + recognized) precede the event; nothing to reenter.
///         - Access control: onlyOwner governs signer rotation + reorg delays.
///           Multisig wiring is [ANT-486] scope (same note as
///           XChainGameRegistry freeze authority); owner is the deployer.
///         - Reorg & finality: attesters sign ONLY finalized home blocks
///           (`homeFinalizedBlock` + `observedAt` bound in the digest);
///           dest enforces a per-home-chain `reorgDelay` AFTER `observedAt`
///           so late reorg evidence can arrive before recognition settles.
///           Unconfigured home chains fail CLOSED (ReorgDelayUnconfigured).
///         - Upgradeability: no proxy in M2-prep; rotation is by owner call.
///         - Fuzz/invariant: threshold counting (distinct members, no double
///           count) + replay set are the money-path properties; see
///           test/M2Attestation.ts.
contract XChainAttester is EIP712, Ownable {
    /* ============ CONSTANTS ============ */

    /// Envelope/attester schema version (M0 M5 lineage, CURRENT_VERSION = 1).
    uint8 public constant CURRENT_VERSION = 1;
    /// Quorum: 3 distinct members of the 5-key attester set.
    uint256 public constant THRESHOLD = 3;
    uint256 public constant ATTESTER_COUNT = 5;

    /// EIP-712 type string for the attestation struct. Field order MUST match
    /// `Attestation` below AND the Rust encoder in
    /// chains/solana/programs/registry/src/xchain_attest.rs.
    bytes32 public constant ATTESTATION_TYPEHASH =
        keccak256(
            "Attestation(bytes32 gameId,string homeChainId,string destChainId,bytes32 licensee,bool isFree,uint256 paidAmount,bytes paidToken,uint8 paidTokenFormat,uint8 receiptVersion,uint8 version,uint256 nonce,uint256 homeFinalizedBlock,uint64 observedAt,bytes32 transportId)"
        );

    /* ============ TYPES ============ */

    struct Attestation {
        bytes32 gameId;
        string homeChainId;
        string destChainId;
        /// Transport-agnostic recipient: EVM address left-padded to 32
        /// bytes, or raw Solana pubkey. Never an EVM20-vs-SOLANA32
        /// comparison — compared only for byte equality (AC-REG-5 lineage).
        bytes32 licensee;
        bool isFree;
        uint256 paidAmount;
        bytes paidToken;
        uint8 paidTokenFormat;
        uint8 receiptVersion;
        uint8 version;
        /// Anti-replay nonce: distinct licenses to the same licensee MUST
        /// use distinct nonces so digests never collide. The consumed-digest
        /// set is the replay backstop, not the nonce.
        uint256 nonce;
        /// Finalized home-chain block the attester observed (NOT latest).
        uint256 homeFinalizedBlock;
        /// Timestamp of that finalized block; reorg delay counts from here.
        uint64 observedAt;
        /// Opaque transport label (e.g. keccak256("wormhole")). Bound, never
        /// interpreted — transport-agnostic by construction.
        bytes32 transportId;
    }

    /* ============ STORAGE ============ */

    /// Current attester set + membership lookup. Rotation bumps `epoch`.
    address[5] public signers;
    mapping(address => bool) public isSigner;
    uint64 public epoch;

    /// Per-home-chain reorg delay in seconds + configured flag (fail-closed).
    mapping(string => uint64) public reorgDelay;
    mapping(string => bool) public reorgDelaySet;

    /// Consumed EIP-712 digests (replay backstop) and recognition records.
    mapping(bytes32 => bool) public consumed;
    mapping(bytes32 => mapping(bytes32 => bool)) public recognized;

    /* ============ EVENTS ============ */

    event AttesterSetRotated(uint64 indexed epoch);
    event ReorgDelaySet(string homeChainId, uint64 delay);
    event AttestationConsumed(
        bytes32 indexed gameId,
        bytes32 indexed licensee,
        uint256 nonce,
        bytes32 digest
    );

    /* ============ ERRORS ============ */

    error InvalidSignerSet();
    error InsufficientSignatures();
    error UnknownSigner(address recovered);
    error DuplicateSignature(address signer);
    error AlreadyConsumed(bytes32 digest);
    error ReorgDelayUnconfigured(string homeChainId);
    error ReorgDelayNotElapsed(uint64 readyAt, uint64 now_);
    error ObservedAtInFuture(uint64 observedAt, uint64 now_);

    /* ============ CONSTRUCTION ============ */

    /// @param initialSigners Exactly 5 distinct non-zero attester addresses.
    constructor(
        address[5] memory initialSigners
    ) EIP712("PGL1-XCHAIN", "1") Ownable(msg.sender) {
        _setSigners(initialSigners);
    }

    /* ============ GOVERNANCE (owner-only; multisig is ANT-486 scope) ============ */

    function rotateSigners(address[5] calldata next) external onlyOwner {
        _setSigners(next);
        unchecked {
            ++epoch;
        }
        emit AttesterSetRotated(epoch);
    }

    /// @notice Configure (or explicitly zero) the reorg delay for a home
    ///         chain. Zero is allowed ONLY via this explicit call — an
    ///         unconfigured chain fails closed in `consume`.
    function setReorgDelay(
        string calldata homeChainId,
        uint64 delay
    ) external onlyOwner {
        reorgDelay[homeChainId] = delay;
        reorgDelaySet[homeChainId] = true;
        emit ReorgDelaySet(homeChainId, delay);
    }

    /* ============ VERIFICATION ============ */

    /// @notice EIP-712 struct hash of an attestation (what the quorum signs
    ///         the preimage of). Exposed for cross-stack test vectors.
    function structHash(Attestation calldata a) public pure returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    ATTESTATION_TYPEHASH,
                    a.gameId,
                    keccak256(bytes(a.homeChainId)),
                    keccak256(bytes(a.destChainId)),
                    a.licensee,
                    a.isFree,
                    a.paidAmount,
                    keccak256(a.paidToken),
                    a.paidTokenFormat,
                    a.receiptVersion,
                    a.version,
                    a.nonce,
                    a.homeFinalizedBlock,
                    a.observedAt,
                    a.transportId
                )
            );
    }

    /// @notice Final EIP-712 digest (0x1901 || domain || structHash) — the
    ///         exact 32 bytes the attesters sign. Domain binds chainId +
    ///         this contract, so cross-chain / cross-contract replay is
    ///         impossible by construction.
    function hashAttestation(
        Attestation calldata a
    ) public view returns (bytes32) {
        return _hashTypedDataV4(structHash(a));
    }

    /// @notice Verify quorum + policy and record recognition. No external
    ///         calls on any path (recognition-only).
    /// @param a The attestation (all fields bound into the signed digest).
    /// @param signatures At least THRESHOLD signatures over `hashAttestation(a)`.
    function consume(
        Attestation calldata a,
        bytes[] calldata signatures
    ) external {
        // Cheap checks first (Gas lens): version + payment rules before any
        // ecrecover. `verifyForDest` enforces AC-ENV-1..3 dest-side even if
        // a quorum member mis-signs an invalid envelope.
        if (a.version > CURRENT_VERSION)
            revert XChainEnvelope.UnsupportedEnvelopeVersion();
        XChainEnvelope.verifyForDest(
            XChainEnvelope.Envelope({
                isFree: a.isFree,
                paidAmount: a.paidAmount,
                paidToken: a.paidToken,
                paidTokenFormat: a.paidTokenFormat,
                receiptVersion: a.receiptVersion,
                version: a.version
            })
        );

        bytes32 digest = _hashTypedDataV4(structHash(a));

        // Replay backstop BEFORE signature work: known digests cost no
        // ecrecovers (Gas lens) and can never be re-consumed.
        if (consumed[digest]) revert AlreadyConsumed(digest);

        if (signatures.length < THRESHOLD) revert InsufficientSignatures();
        uint256 distinct = 0;
        // Bounded loop (5 max) over calldata; duplicates/unknowns revert
        // instead of being silently skipped (fail-closed threshold).
        address[5] memory seen;
        for (uint256 i = 0; i < signatures.length; i++) {
            (address recovered, , ) = ECDSA.tryRecover(
                digest,
                signatures[i]
            );
            if (!isSigner[recovered]) revert UnknownSigner(recovered);
            bool dup = false;
            for (uint256 j = 0; j < distinct; j++) {
                if (seen[j] == recovered) {
                    dup = true;
                    break;
                }
            }
            if (dup) revert DuplicateSignature(recovered);
            seen[distinct] = recovered;
            unchecked {
                ++distinct;
            }
            if (distinct == THRESHOLD) break;
        }
        if (distinct < THRESHOLD) revert InsufficientSignatures();

        // Finality/reorg policy AFTER quorum (quorum is the expensive part
        // to re-run off-chain; policy is a cheap timestamp check).
        if (!reorgDelaySet[a.homeChainId])
            revert ReorgDelayUnconfigured(a.homeChainId);
        if (a.observedAt > block.timestamp)
            revert ObservedAtInFuture(a.observedAt, uint64(block.timestamp));
        uint64 readyAt = a.observedAt + reorgDelay[a.homeChainId];
        if (block.timestamp < readyAt)
            revert ReorgDelayNotElapsed(readyAt, uint64(block.timestamp));

        // Effects (no external calls — nothing to reenter).
        consumed[digest] = true;
        recognized[a.gameId][a.licensee] = true;
        emit AttestationConsumed(a.gameId, a.licensee, a.nonce, digest);
    }

    /// @notice Dest-side read: is (gameId, licensee) recognized here?
    function isRecognized(
        bytes32 gameId,
        bytes32 licensee
    ) external view returns (bool) {
        return recognized[gameId][licensee];
    }

    /* ============ INTERNALS ============ */

    function _setSigners(address[5] memory next) internal {
        for (uint256 i = 0; i < ATTESTER_COUNT; i++) {
            if (next[i] == address(0)) revert InvalidSignerSet();
            for (uint256 j = i + 1; j < ATTESTER_COUNT; j++) {
                if (next[i] == next[j]) revert InvalidSignerSet();
            }
        }
        // Clear old membership BEFORE installing the new set so a rotated-out
        // key never stays valid (no stale-membership window).
        address[5] memory prev = signers;
        for (uint256 i = 0; i < ATTESTER_COUNT; i++) {
            if (prev[i] != address(0)) isSigner[prev[i]] = false;
        }
        signers = next;
        for (uint256 i = 0; i < ATTESTER_COUNT; i++) {
            isSigner[next[i]] = true;
        }
    }
}
