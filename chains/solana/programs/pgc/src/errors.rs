use anchor_lang::prelude::*;

#[error_code]
pub enum Pgc1Error {
    #[msg("Game ID must not be empty")]
    EmptyGameId,

    #[msg("Metadata URI must not be empty")]
    EmptyMetadataUri,

    #[msg("Invalid publisher")]
    InvalidPublisher,

    #[msg("Invalid minter")]
    InvalidMinter,

    #[msg("Invalid receiver")]
    InvalidReceiver,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("String too long")]
    StringTooLong,

    #[msg("License account mismatch")]
    LicenseAccountMismatch,
}
