use anchor_lang::prelude::*;

#[event]
pub struct StoreInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub treasury: Pubkey,
}

#[event]
pub struct PlatformFeeUpdated {
    pub platform_fee_bps: u16,
}

#[event]
pub struct DefaultReferralUpdated {
    pub default_referral_bps: u16,
}

#[event]
pub struct MaxReferralUpdated {
    pub max_referral_bps: u16,
}

#[event]
pub struct AuthorizedSourceProgramAdded {
    pub program_id: Pubkey,
}

#[event]
pub struct AuthorizedSourceProgramUpdated {
    pub program_id: Pubkey,
    pub active: bool,
}

#[event]
pub struct AuthorizedRegistryProgramAdded {
    pub program_id: Pubkey,
}

#[event]
pub struct AuthorizedRegistryProgramUpdated {
    pub program_id: Pubkey,
    pub active: bool,
}

#[event]
pub struct PaymentTokenAdded {
    pub mint: Pubkey,
}

#[event]
pub struct PaymentTokenUpdated {
    pub mint: Pubkey,
    pub active: bool,
}

#[event]
pub struct GameStoreConfigInitialized {
    pub game: Pubkey,
    pub active: bool,
}

#[event]
pub struct GameStoreActiveUpdated {
    pub game: Pubkey,
    pub active: bool,
}

#[event]
pub struct GamePaymentOptionSet {
    pub game: Pubkey,
    pub mint: Pubkey,
    pub base_price: u64,
    pub active: bool,
}

#[event]
pub struct GamePaymentOptionRemoved {
    pub game: Pubkey,
    pub mint: Pubkey,
}

#[event]
pub struct DiscountSet {
    pub game: Pubkey,
    pub discount_bps: Option<u16>,
    pub discount_starts_at: Option<i64>,
    pub discount_expires_at: Option<i64>,
}

#[event]
pub struct DiscountCleared {
    pub game: Pubkey,
}

#[event]
pub struct ReferralBpsUpdated {
    pub game: Pubkey,
    pub referral_bps: Option<u16>,
}

#[event]
pub struct GamePurchased {
    pub buyer: Pubkey,
    pub game: Pubkey,
    pub payment_mint: Pubkey,
    pub paid_amount: u64,
    pub final_price: u64,
    pub referral_bps_applied: u16,
}

#[event]
pub struct PurchaseReceiptCreated {
    pub buyer: Pubkey,
    pub game: Pubkey,
}

#[event]
pub struct StoreActorUpdated {
    pub old_store_actor: Pubkey,
    pub new_store_actor: Pubkey,
}
