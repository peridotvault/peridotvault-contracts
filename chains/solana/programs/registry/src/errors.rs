use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Game ID must not be empty")]
    EmptyGameId,

    #[msg("Game ID is too long")]
    GameIdTooLong,

    #[msg("Invalid contract address")]
    InvalidContractAddress,

    #[msg("Invalid governance address")]
    InvalidGovernance,

    #[msg("Invalid treasury address")]
    InvalidTreasury,

    #[msg("Invalid factory address")]
    InvalidFactory,

    #[msg("Invalid publisher address")]
    InvalidPublisher,

    #[msg("Invalid admin address")]
    InvalidAdmin,

    #[msg("Invalid fee exemption address")]
    InvalidFeeExemptionAccount,

    #[msg("Invalid registration fee token")]
    InvalidRegistrationFeeToken,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Game is already registered")]
    GameAlreadyRegistered,

    #[msg("Game was not found")]
    GameNotFound,

    #[msg("Invalid game status")]
    InvalidStatus,

    #[msg("Provided game ID does not match PGC canonical game ID")]
    GameIdMismatch,

    #[msg("Provided publisher does not match PGC canonical publisher")]
    PublisherMismatch,

    #[msg("Registry game limit reached")]
    RegistryFull,

    #[msg("Registry admin limit reached")]
    AdminListFull,

    #[msg("Registry fee exemption limit reached")]
    FeeExemptionListFull,

    #[msg("Missing required fee accounts")]
    MissingFeeAccounts,

    #[msg("Invalid fee payer token account")]
    InvalidFeePayerTokenAccount,

    #[msg("Invalid treasury token account")]
    InvalidTreasuryTokenAccount,

    #[msg("Registration fee mint mismatch")]
    RegistrationFeeMintMismatch,
}
