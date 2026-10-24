use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Payment token not allowed")]
    PaymentTokenNotAllowed,

    #[msg("Payment token disabled")]
    PaymentTokenDisabled,

    #[msg("Invalid fee amount")]
    InvalidFeeAmount,

    #[msg("Registration fee not satisfied")]
    RegistrationFeeNotSatisfied,

    #[msg("Game already registered")]
    GameAlreadyRegistered,

    #[msg("Invalid game id")]
    InvalidGameId,

    #[msg("Game not found")]
    GameNotFound,

    #[msg("Invalid status transition")]
    InvalidStatusTransition,

    #[msg("Invalid expiry")]
    InvalidExpiry,

    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,

    #[msg("Invalid publish grant account")]
    InvalidPublishGrantAccount,

    #[msg("Invalid treasury")]
    InvalidTreasury,

    #[msg("Invalid PGL-1 program")]
    InvalidPgl1Program,

    #[msg("Invalid PGL-1 config account")]
    InvalidPgl1Config,

    #[msg("Invalid store program")]
    InvalidStoreProgram,

    #[msg("Invalid price")]
    InvalidPrice,

    #[msg("Missing store accounts for paid game")]
    MissingStoreAccounts,

    #[msg("Game not closable (must be Suspended or Banned)")]
    GameNotClosable,

    #[msg("Game mismatch (registry_game.game != game)")]
    GameMismatch,

    #[msg("Insufficient fee balance in publisher payment account")]
    InsufficientFeeBalance,
}

#[error_code]
pub enum XchainErrorCode {
    #[msg("Unknown chain id")]
    UnknownChainId,

    #[msg("Invalid publisher bytes")]
    InvalidPublisher,

    #[msg("Empty local id")]
    EmptyLocalId,

    #[msg("Local id too long")]
    LocalIdTooLong,

    #[msg("Invalid local id charset")]
    InvalidLocalIdCharset,

    #[msg("Invalid program id")]
    InvalidProgramId,

    #[msg("Entry already exists")]
    EntryAlreadyExists,

    #[msg("Entry not found")]
    EntryNotFound,

    #[msg("Invalid status transition")]
    InvalidXchainTransition,

    #[msg("Entry is frozen")]
    EntryFrozen,

    #[msg("Update already pending")]
    UpdateAlreadyPending,

    #[msg("No pending update")]
    NoPendingUpdate,

    #[msg("Update not yet effective")]
    UpdateNotEffective,

    #[msg("Zero timelock delay")]
    ZeroTimelock,

    #[msg("Unsupported version")]
    UnsupportedXchainVersion,

    #[msg("Paid path with zero settlement")]
    PaidAmountZeroOnPaidPath,

    #[msg("Free path with nonzero settlement")]
    PaidAmountNonZeroOnFreePath,

    #[msg("Receipt/envelope version mismatch")]
    XchainVersionMismatch,

    #[msg("gameId argument is not the canonical derivation")]
    GameIdMismatch,

    #[msg("Free path with non-empty paid token")]
    PaidTokenNonEmptyOnFreePath,

    #[msg("Invalid paid token format")]
    InvalidTokenFormat,

    #[msg("Unknown attester (recovered signer outside the attester set)")]
    UnknownAttester,

    #[msg("Duplicate attester signature toward the quorum")]
    DuplicateAttesterSignature,

    #[msg("Attestation quorum threshold not met")]
    AttestationThresholdNotMet,

    #[msg("Attestation digest already consumed")]
    AttestationAlreadyConsumed,

    #[msg("Invalid attestation signature")]
    InvalidAttestationSignature,

    #[msg("Attestation reorg delay not elapsed")]
    AttestationReorgDelayNotElapsed,

    #[msg("Attestation observation timestamp is in the future")]
    AttestationObservedAtInFuture,
}
