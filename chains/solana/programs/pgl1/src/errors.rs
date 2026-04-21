use anchor_lang::prelude::*;

#[error_code]
pub enum PglError {
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Authorized actor is inactive")]
    AuthorizedActorInactive,

    #[msg("Insufficient create game fee")]
    InsufficientCreateGameFee,

    #[msg("Invalid game id")]
    InvalidGameId,

    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,

    #[msg("Game already exists")]
    GameAlreadyExists,

    #[msg("License already exists")]
    LicenseAlreadyExists,

    #[msg("License not found")]
    LicenseNotFound,

    #[msg("Invalid expiry")]
    InvalidExpiry,
}
