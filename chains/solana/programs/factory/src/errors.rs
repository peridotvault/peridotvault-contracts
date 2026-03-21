use anchor_lang::prelude::*;

#[error_code]
pub enum FactoryError {
    #[msg("Game ID must not be empty")]
    EmptyGameId,

    #[msg("Metadata URI must not be empty")]
    EmptyMetadataUri,

    #[msg("Game ID is too long")]
    GameIdTooLong,

    #[msg("Metadata URI is too long")]
    MetadataUriTooLong,

    #[msg("Invalid governance address")]
    InvalidGovernance,

    #[msg("Invalid registry address")]
    InvalidRegistry,

    #[msg("Invalid game store address")]
    InvalidGameStore,

    #[msg("Invalid mint PDA")]
    InvalidMint,

    #[msg("Unauthorized")]
    Unauthorized,
}
