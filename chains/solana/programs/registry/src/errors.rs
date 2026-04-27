use quasar_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    Unauthorized,
    PaymentTokenNotAllowed,
    PaymentTokenDisabled,
    InvalidFeeAmount,
    RegistrationFeeNotSatisfied,
    GameAlreadyRegistered,
    InvalidGameId,
    GameNotFound,
    InvalidStatusTransition,
    InvalidExpiry,
    InvalidMetadataUri,
    InvalidPublishGrantAccount,
    InvalidTreasury,
    InvalidPgl1Program,
    InvalidPgl1Config,
    InvalidStoreProgram,
    InvalidPrice,
    MissingStoreAccounts,
    GameNotClosable,
    GameMismatch,
    InsufficientFeeBalance,
}
