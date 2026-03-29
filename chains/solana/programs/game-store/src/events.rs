use anchor_lang::prelude::*;

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
pub struct PriceUpdated {
    pub game: Pubkey,
    pub price: u64,
    pub currency: Pubkey,
}

#[event]
pub struct PlatformFeeUpdated {
    pub old_bps: u16,
    pub new_bps: u16,
}

#[event]
pub struct TreasuryUpdated {
    pub old_treasury: Pubkey,
    pub new_treasury: Pubkey,
}

#[event]
pub struct Withdrawal {
    pub publisher: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
}
