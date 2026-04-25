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
}
