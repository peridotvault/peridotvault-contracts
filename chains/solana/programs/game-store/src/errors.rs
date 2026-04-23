use anchor_lang::prelude::*;

#[error_code]
pub enum StoreError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid platform fee bps")]
    InvalidPlatformFeeBps,
    #[msg("Invalid default referral bps")]
    InvalidDefaultReferralBps,
    #[msg("Invalid max referral bps")]
    InvalidMaxReferralBps,
    #[msg("Referral above max")]
    ReferralAboveMax,
    #[msg("Source program not authorized")]
    SourceProgramNotAuthorized,
    #[msg("Registry program not authorized")]
    RegistryProgramNotAuthorized,
    #[msg("Payment token not allowed")]
    PaymentTokenNotAllowed,
    #[msg("Payment token disabled")]
    PaymentTokenDisabled,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Price not found")]
    PriceNotFound,
    #[msg("Game not active in store")]
    StoreGameInactive,
    #[msg("Game not active in registry")]
    GameNotActive,
    #[msg("Game not registered")]
    GameNotRegistered,
    #[msg("Already owned")]
    AlreadyOwned,
    #[msg("Invalid discount bps")]
    InvalidDiscountBps,
    #[msg("Invalid discount window")]
    InvalidDiscountWindow,
    #[msg("Invalid referral bps")]
    InvalidReferralBps,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid payment amount")]
    InvalidPaymentAmount,
    #[msg("Unsupported source game owner")]
    UnsupportedSourceGameOwner,
    #[msg("Registry game mismatch")]
    RegistryGameMismatch,
    #[msg("Payment failed")]
    PaymentFailed,
    #[msg("License mint failed")]
    LicenseMintFailed,
    #[msg("Missing referrer token account")]
    MissingReferrerTokenAccount,
    #[msg("Invalid referrer token account")]
    InvalidReferrerTokenAccount,
    #[msg("Invalid treasury")]
    InvalidTreasury,
    #[msg("Invalid store actor")]
    InvalidStoreActor,
}
