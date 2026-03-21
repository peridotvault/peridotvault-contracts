use anchor_lang::prelude::*;

#[event]
pub struct StoreInitialized {
    pub governance: Pubkey,
    pub treasury: Pubkey,
    pub registry: Pubkey,
    pub platform_fee_bps: u16,
}

#[event]
pub struct PriceSet {
    pub game_id: String,
    pub publisher: Pubkey,
    pub price: u64,
    pub currency: Pubkey,
}

#[event]
pub struct DiscountSet {
    pub game_id: String,
    pub publisher: Pubkey,
    pub discount_bps: u16,
}

#[event]
pub struct GamePurchased {
    pub game_id: String,
    pub buyer: Pubkey,
    pub publisher: Pubkey,
    pub currency: Pubkey,
    pub final_price: u64,
    pub platform_fee: u64,
    pub publisher_revenue: u64,
}

#[event]
pub struct PlatformFeeUpdated {
    pub platform_fee_bps: u16,
}

#[event]
pub struct GovernanceUpdated {
    pub old_governance: Pubkey,
    pub new_governance: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub old_treasury: Pubkey,
    pub new_treasury: Pubkey,
}

#[event]
pub struct PublisherWithdrawn {
    pub publisher: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
}

#[event]
pub struct NativeSolPublisherWithdrawn {
    pub publisher: Pubkey,
    pub amount: u64,
}
