use anchor_lang::prelude::*;

#[error_code]
pub enum GameStoreError {
    #[msg("Game ID must not be empty")]
    EmptyGameId,

    #[msg("Game ID is too long")]
    GameIdTooLong,

    #[msg("Invalid governance address")]
    InvalidGovernance,

    #[msg("Invalid treasury address")]
    InvalidTreasury,

    #[msg("Invalid registry address")]
    InvalidRegistry,

    #[msg("Invalid currency address")]
    InvalidCurrency,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Game was not found in registry")]
    GameNotFound,

    #[msg("Game is not approved")]
    GameNotApproved,

    #[msg("Price config was not found")]
    PriceConfigNotFound,

    #[msg("Discount basis points are invalid")]
    InvalidDiscountBps,

    #[msg("Platform fee basis points are invalid")]
    InvalidPlatformFeeBps,

    #[msg("Publisher balance not found")]
    PublisherBalanceNotFound,

    #[msg("Publisher balance is zero")]
    EmptyPublisherBalance,

    #[msg("Registry game contract mismatch")]
    ContractAddressMismatch,

    #[msg("Registry game publisher mismatch")]
    PublisherMismatch,

    #[msg("Duplicate purchase is not allowed")]
    AlreadyOwnsValidLicense,

    #[msg("Invalid payment mint")]
    InvalidPaymentMint,

    #[msg("Invalid buyer token account")]
    InvalidBuyerTokenAccount,

    #[msg("Invalid treasury token account")]
    InvalidTreasuryTokenAccount,

    #[msg("Invalid treasury account")]
    InvalidTreasuryAccount,

    #[msg("Invalid store vault token account")]
    InvalidStoreVaultTokenAccount,

    #[msg("Store price config limit reached")]
    PriceConfigLimitReached,

    #[msg("Store publisher balance limit reached")]
    PublisherBalanceLimitReached,

    #[msg("Invalid license account")]
    InvalidLicenseAccount,

    #[msg("Use the native SOL instruction for this game price")]
    NativeSolRequiresDedicatedInstruction,

    #[msg("Store account does not have enough SOL escrow")]
    InsufficientStoreLamports,
}
