// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title XChainGameRegistry — PGC1 M1 canonical game registry (ANT-490)
/// @notice Implements parent ANT-485 `spec` §3.2 (registry schema + lifecycle)
///         and §3.3 (timelock schema). New contract on purpose: M0-versioned
///         structs in PeridotRegistry/PGC1 are EXTENDED, never reshaped
///         (spec §7) — PeridotRegistry.sol is intentionally untouched.
///
///         Q2 answer (spec §9): `pendingUpdate` lives INLINE on the entry
///         (single-read path for the attester, NFR-6). Same recommendation
///         is mirrored on Solana (`XchainGameEntry.pending`, see
///         chains/solana/programs/registry/src/state.rs).
///
///         Lenses: Access control (owner-only governance; freeze authority =
///         owner until [ANT-486] wires multisig — see EMERGENCY note below);
///         Checks-effects-interactions (no external calls on any path, all
///         state changes precede events); Gas (calldata args, cheap length
///         checks short-circuit before hashing).
contract XChainGameRegistry is Ownable {
    /// M1 schema version (M0 M5 lineage, CURRENT_VERSION = 1).
    uint8 public constant CURRENT_VERSION = 1;
    /// Max localId length in bytes (spec §3.1: 1..128).
    uint256 public constant MAX_LOCAL_ID_LEN = 128;

    /// Lifecycle §3.2: PROPOSED → ACTIVE → DEPRECATED → FROZEN (terminal).
    enum XStatus {
        PROPOSED,
        ACTIVE,
        DEPRECATED,
        FROZEN
    }

    /// bytes+format discipline (spec §3.2): never compare raw bytes across
    /// formats (AC-REG-5).
    enum IdFormat {
        EVM20,
        SOLANA32
    }

    struct PendingUpdate {
        bytes proposedProgramId;
        IdFormat proposedFormat;
        uint64 proposedAt;
        uint64 effectiveAt;
        uint64 timelockDelay;
        bytes proposer;
    }

    struct XEntry {
        string chainId;
        bytes programId;
        IdFormat format;
        uint8 version;
        XStatus status;
        bytes32 gameId;
        uint64 registeredAt;
        bool hasPending;
        PendingUpdate pending;
    }

    /* ============ STORAGE ============ */

    mapping(bytes32 => XEntry) private _entries;
    mapping(bytes32 => bool) private _exists;
    /// chainId allowlist — spec §3.1 normative table (exact byte equality).
    mapping(string => bool) public supportedChain;

    /* ============ EVENTS (§3.2: every transition emits) ============ */

    event GameProposed(bytes32 indexed gameId, string chainId);
    event GameActivated(bytes32 indexed gameId, uint64 registeredAt);
    event GameDeprecated(bytes32 indexed gameId);
    /// Distinct emergency-freeze event (AC-TIME-3): never reuse the
    /// deprecate/freeze event shape so indexers can tell the bypass apart.
    event EmergencyFreeze(bytes32 indexed gameId, address indexed freezer);
    event ProposalCancelled(bytes32 indexed gameId);
    event UpdateProposed(
        bytes32 indexed gameId,
        uint64 effectiveAt,
        uint64 timelockDelay
    );
    event UpdateCancelled(bytes32 indexed gameId);
    event UpdateExecuted(bytes32 indexed gameId);

    /* ============ ERRORS (negatives assert these specifically) ============ */

    error UnknownChainId();
    error InvalidPublisher();
    error EmptyLocalId();
    error LocalIdTooLong();
    error InvalidLocalIdCharset();
    error InvalidProgramId();
    error GameAlreadyExists();
    error EntryNotFound();
    error InvalidTransition();
    error EntryFrozen();
    error UpdateAlreadyPending();
    error NoPendingUpdate();
    error UpdateNotEffective();
    error ZeroTimelock();
    error UnsupportedVersion();

    constructor() Ownable(msg.sender) {
        supportedChain["eip155:31337"] = true;
        supportedChain["eip155:84532"] = true;
        supportedChain["eip155:4202"] = true;
        supportedChain["solana:localnet"] = true;
        supportedChain[
            "solana:EtWBMkgFUSkMhvZSGQi4uAiEE8RjqiB1MwWXEaAu34"
        ] = true;
    }

    /* ============ GAMEID (§3.1 normative) ============ */

    /// gameId = keccak256(abi.encode(string chainId, bytes publisher, string localId))
    function deriveGameId(
        string calldata chainId,
        bytes calldata publisher,
        string calldata localId
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(chainId, publisher, localId));
    }

    /* ============ REGISTRATION ============ */

    /// Propose a new record. Starts at PROPOSED, never directly ACTIVE.
    /// gameId is derived on-chain so registration validates the inputs
    /// (AC-GAMEID-4: empty localId / unknown chainId fail here).
    function proposeGame(
        string calldata chainId,
        bytes calldata publisher,
        string calldata localId,
        bytes calldata programId,
        IdFormat format
    ) external onlyOwner returns (bytes32 gameId) {
        if (!supportedChain[chainId]) revert UnknownChainId();
        _checkPublisher(publisher, format);
        _checkLocalId(localId);
        _checkProgramId(programId, format);

        gameId = deriveGameId(chainId, publisher, localId);
        if (_exists[gameId]) revert GameAlreadyExists();

        XEntry storage e = _entries[gameId];
        e.chainId = chainId;
        e.programId = programId;
        e.format = format;
        e.version = CURRENT_VERSION;
        e.status = XStatus.PROPOSED;
        e.gameId = gameId;
        e.registeredAt = 0;
        _exists[gameId] = true;

        emit GameProposed(gameId, chainId);
    }

    function activateGame(bytes32 gameId) external onlyOwner {
        XEntry storage e = _mustGet(gameId);
        if (e.status != XStatus.PROPOSED) revert InvalidTransition();
        e.status = XStatus.ACTIVE;
        e.registeredAt = uint64(block.timestamp);
        emit GameActivated(gameId, e.registeredAt);
    }

    function deprecateGame(bytes32 gameId) external onlyOwner {
        XEntry storage e = _mustGet(gameId);
        if (e.status != XStatus.ACTIVE) revert InvalidTransition();
        e.status = XStatus.DEPRECATED;
        emit GameDeprecated(gameId);
    }

    /// Emergency freeze bypasses the timelock (AC-TIME-3): immediate
    /// ACTIVE/DEPRECATED → FROZEN with a DISTINCT event. One-way: FROZEN is
    /// terminal, re-activation always fails (AC-REG-3).
    /// EMERGENCY NOTE: freeze authority is `owner` in M1. Multisig/timelock
    /// wiring is [ANT-486] scope (spec §3.3 design constraint) — flagged for
    /// Tobias review (ANT-491).
    function freezeGame(bytes32 gameId) external onlyOwner {
        XEntry storage e = _mustGet(gameId);
        if (e.status != XStatus.ACTIVE && e.status != XStatus.DEPRECATED)
            revert InvalidTransition();
        e.status = XStatus.FROZEN;
        emit EmergencyFreeze(gameId, msg.sender);
    }

    /// Reject/cancel a PROPOSED record: record closed, terminal.
    function cancelProposal(bytes32 gameId) external onlyOwner {
        XEntry storage e = _mustGet(gameId);
        if (e.status != XStatus.PROPOSED) revert InvalidTransition();
        delete _entries[gameId];
        delete _exists[gameId];
        emit ProposalCancelled(gameId);
    }

    /* ============ TIMELOCK (§3.3) ============ */

    /// Single-flight: proposing while one is in flight MUST fail
    /// (AC-TIME-2). `timelockDelay == 0` is rejected (short ≠ 0, §5);
    /// localnet fixtures use a short but enforced delay (e.g. 60s).
    function proposeUpdate(
        bytes32 gameId,
        bytes calldata newProgramId,
        uint64 timelockDelay
    ) external onlyOwner {
        XEntry storage e = _mustGet(gameId);
        if (e.status == XStatus.FROZEN) revert EntryFrozen();
        if (e.status != XStatus.ACTIVE && e.status != XStatus.DEPRECATED)
            revert InvalidTransition();
        if (e.hasPending) revert UpdateAlreadyPending();
        if (timelockDelay == 0) revert ZeroTimelock();
        _checkProgramId(newProgramId, e.format);

        e.hasPending = true;
        e.pending = PendingUpdate({
            proposedProgramId: newProgramId,
            proposedFormat: e.format,
            proposedAt: uint64(block.timestamp),
            effectiveAt: uint64(block.timestamp) + timelockDelay,
            timelockDelay: timelockDelay,
            proposer: abi.encodePacked(msg.sender)
        });
        emit UpdateProposed(gameId, e.pending.effectiveAt, timelockDelay);
    }

    function cancelUpdate(bytes32 gameId) external onlyOwner {
        XEntry storage e = _mustGet(gameId);
        if (!e.hasPending) revert NoPendingUpdate();
        delete e.pending;
        e.hasPending = false;
        emit UpdateCancelled(gameId);
    }

    /// Execution MUST revert before effectiveAt (AC-TIME-1), even with the
    /// short localnet delay. Refuses FROZEN entries fail-closed.
    function executeUpdate(bytes32 gameId) external onlyOwner {
        XEntry storage e = _mustGet(gameId);
        if (e.status == XStatus.FROZEN) revert EntryFrozen();
        if (!e.hasPending) revert NoPendingUpdate();
        if (block.timestamp < e.pending.effectiveAt)
            revert UpdateNotEffective();
        e.programId = e.pending.proposedProgramId;
        delete e.pending;
        e.hasPending = false;
        emit UpdateExecuted(gameId);
    }

    /* ============ READS (fail-closed) ============ */

    /// Fail-closed read: unregistered gameId → EntryNotFound (never
    /// default-ACTIVE); version > CURRENT → UnsupportedVersion (M5 lineage).
    function getEntry(
        bytes32 gameId
    ) external view returns (XEntry memory) {
        if (!_exists[gameId]) revert EntryNotFound();
        XEntry memory e = _entries[gameId];
        if (e.version > CURRENT_VERSION) revert UnsupportedVersion();
        return e;
    }

    /// Attester gate: MUST refuse FROZEN records (§3.2). DEPRECATED stays
    /// servable (dual-accept window); warn-by-event happens at consumption
    /// time in [ANT-487], views cannot emit.
    function requireAuthoritative(bytes32 gameId) external view returns (XEntry memory) {
        if (!_exists[gameId]) revert EntryNotFound();
        XEntry memory e = _entries[gameId];
        if (e.version > CURRENT_VERSION) revert UnsupportedVersion();
        if (e.status == XStatus.FROZEN) revert EntryFrozen();
        return e;
    }

    /* ============ INTERNALS ============ */

    function _mustGet(bytes32 gameId) internal view returns (XEntry storage e) {
        if (!_exists[gameId]) revert EntryNotFound();
        e = _entries[gameId];
        if (e.version > CURRENT_VERSION) revert UnsupportedVersion();
    }

    function _checkPublisher(
        bytes calldata publisher,
        IdFormat format
    ) internal pure {
        uint256 want = format == IdFormat.EVM20 ? 20 : 32;
        if (publisher.length != want) revert InvalidPublisher();
    }

    function _checkProgramId(
        bytes calldata programId,
        IdFormat format
    ) internal pure {
        uint256 want = format == IdFormat.EVM20 ? 20 : 32;
        if (programId.length != want) revert InvalidProgramId();
    }

    /// 1..128 bytes, printable ASCII only (0x21..0x7E). This excludes ALL
    /// whitespace (stricter than "whitespace runs", keeps EVM/Solana string
    /// handling byte-identical per the §5 ASCII-only rule).
    function _checkLocalId(string calldata localId) internal pure {
        bytes memory b = bytes(localId);
        if (b.length == 0) revert EmptyLocalId();
        if (b.length > MAX_LOCAL_ID_LEN) revert LocalIdTooLong();
        for (uint256 i = 0; i < b.length; i++) {
            uint8 c = uint8(b[i]);
            if (c < 0x21 || c > 0x7E) revert InvalidLocalIdCharset();
        }
    }
}
