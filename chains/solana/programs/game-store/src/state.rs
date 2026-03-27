use anchor_lang::prelude::*;

#[account]
pub struct PriceAccount {
    pub bump: u8,
    pub game: Pubkey,
    pub price: u64,
    pub currency: Pubkey,
    pub discount_bps: u16,
}

impl PriceAccount {
    pub const SPACE: usize = 8 + 1 + 32 + 8 + 32 + 2;
    
    pub fn final_price(&self) -> u64 {
        let discount = (u128::from(self.price) * u128::from(self.discount_bps)) / 10_000;
        (u128::from(self.price) - discount) as u64
    }
}

#[account]
pub struct PublisherBalanceAccount {
    pub bump: u8,
    pub publisher: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
}

impl PublisherBalanceAccount {
    pub const SPACE: usize = 8 + 1 + 32 + 32 + 8;
}

#[account]
pub struct AffiliateAccount {
    pub bump: u8,
    pub game: Pubkey,
    pub affiliate: Pubkey,
    pub share_bps: u16,
}

impl AffiliateAccount {
    pub const SPACE: usize = 8 + 1 + 32 + 32 + 2;
}

#[account]
pub struct SubscriptionAccount {
    pub bump: u8,
    pub game: Pubkey,
    pub price: u64,
    pub duration: i64,
    pub enabled: bool,
}

impl SubscriptionAccount {
    pub const SPACE: usize = 8 + 1 + 32 + 8 + 8 + 1;
}

#[account]
pub struct StoreConfig {
    pub bump: u8,
    pub treasury: Pubkey,
    pub governance: Pubkey,
    pub platform_fee_bps: u16,
}

impl StoreConfig {
    pub const SPACE: usize = 8 + 1 + 32 + 32 + 2;
}
