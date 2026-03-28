use anchor_lang::prelude::*;

#[error_code]
pub enum PgcError {
    #[msg("Game already exists")]
    GameAlreadyExists,
    #[msg("License already exists")]
    LicenseAlreadyExists,
    #[msg("License has expired")]
    LicenseExpired,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid game ID")]
    InvalidGameId,
    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,
    #[msg("Registry call failed")]
    RegistryCallFailed,
    #[msg("Store call failed")]
    StoreCallFailed,
    #[msg("Invalid minter account")]
    InvalidMinter,
}
