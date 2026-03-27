use anchor_lang::prelude::*;

#[error_code]
pub enum GameStoreError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid currency")]
    InvalidCurrency,
    #[msg("Invalid discount BPS")]
    InvalidDiscountBps,
    #[msg("Invalid platform fee BPS")]
    InvalidPlatformFeeBps,
    #[msg("Empty publisher balance")]
    EmptyPublisherBalance,
    #[msg("Invalid governance address")]
    InvalidGovernance,
    #[msg("Invalid treasury address")]
    InvalidTreasury,
}
