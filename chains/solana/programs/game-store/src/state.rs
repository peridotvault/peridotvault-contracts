use anchor_lang::prelude::*;

pub const BPS_DENOMINATOR: u64 = 10_000;
pub const PLATFORM_FEE_BPS_MAX: u16 = 10_000;
pub const MAX_REFERRAL_BPS_HARD_CAP: u16 = 5_000;

#[account]
pub struct StoreConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub platform_fee_bps: u16,
    pub default_referral_bps: u16,
    pub max_referral_bps: u16,
    pub store_actor: Pubkey,
    pub bump: u8,
}

impl StoreConfig {
    pub const LEN: usize = 32 + 32 + 2 + 2 + 2 + 32 + 1;
}

#[account]
pub struct AuthorizedSourceProgram {
    pub program_id: Pubkey,
    pub active: bool,
    pub bump: u8,
}

impl AuthorizedSourceProgram {
    pub const LEN: usize = 32 + 1 + 1;
}

#[account]
pub struct AuthorizedRegistryProgram {
    pub program_id: Pubkey,
    pub active: bool,
    pub bump: u8,
}

impl AuthorizedRegistryProgram {
    pub const LEN: usize = 32 + 1 + 1;
}

#[account]
pub struct AcceptedPaymentToken {
    pub mint: Pubkey,
    pub active: bool,
    pub bump: u8,
}

impl AcceptedPaymentToken {
    pub const LEN: usize = 32 + 1 + 1;
}

#[account]
pub struct GameStoreConfig {
    pub game: Pubkey,
    pub active: bool,
    pub referral_bps: Option<u16>,
    pub discount_bps: Option<u16>,
    pub discount_starts_at: Option<i64>,
    pub discount_expires_at: Option<i64>,
    pub bump: u8,
}

impl GameStoreConfig {
    pub const LEN: usize = 32 + 1 + 3 + 3 + 9 + 9 + 1;
}

#[account]
pub struct GamePaymentOption {
    pub game: Pubkey,
    pub mint: Pubkey,
    pub base_price: u64,
    pub active: bool,
    pub bump: u8,
}

impl GamePaymentOption {
    pub const LEN: usize = 32 + 32 + 8 + 1 + 1;
}

#[account]
pub struct PurchaseReceipt {
    pub buyer: Pubkey,
    pub game: Pubkey,
    pub payment_mint: Pubkey,
    pub paid_amount: u64,
    pub final_price: u64,
    pub referral_bps_applied: u16,
    pub purchased_at: i64,
    pub bump: u8,
}

impl PurchaseReceipt {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 2 + 8 + 1;
}
