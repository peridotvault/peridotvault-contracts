use quasar_lang::prelude::*;

pub struct StoreInitialized {
    pub authority: Address,
    pub treasury: Address,
}

pub struct TreasuryUpdated {
    pub treasury: Address,
}

pub struct PlatformFeeUpdated {
    pub platform_fee_bps: u16,
}

pub struct DefaultReferralUpdated {
    pub default_referral_bps: u16,
}

pub struct MaxReferralUpdated {
    pub max_referral_bps: u16,
}

pub struct AuthorizedProgramAdded {
    pub program_id: Address,
    pub role: u8,
}

pub struct AuthorizedProgramUpdated {
    pub program_id: Address,
    pub active: bool,
    pub role: u8,
}

pub struct PaymentTokenAdded {
    pub mint: Address,
}

pub struct PaymentTokenUpdated {
    pub mint: Address,
    pub active: bool,
}

pub struct GameStoreConfigInitialized {
    pub game: Address,
    pub active: bool,
}

pub struct GameStoreActiveUpdated {
    pub game: Address,
    pub active: bool,
}

pub struct GamePaymentOptionSet {
    pub game: Address,
    pub mint: Address,
    pub base_price: u64,
    pub active: bool,
}

pub struct GamePaymentOptionRemoved {
    pub game: Address,
    pub mint: Address,
}

pub struct DiscountSet {
    pub game: Address,
    pub discount_bps_present: bool,
    pub discount_bps: u16,
    pub discount_starts_at_present: bool,
    pub discount_starts_at: i64,
    pub discount_expires_at_present: bool,
    pub discount_expires_at: i64,
}

pub struct DiscountCleared {
    pub game: Address,
}

pub struct ReferralBpsUpdated {
    pub game: Address,
    pub referral_bps_present: bool,
    pub referral_bps: u16,
}

pub struct GamePurchased {
    pub buyer: Address,
    pub game: Address,
    pub payment_mint: Address,
    pub paid_amount: u64,
    pub final_price: u64,
    pub referrer: Address,
    pub referral_bps_applied: u16,
}

pub struct PurchaseReceiptCreated {
    pub buyer: Address,
    pub game: Address,
    pub referrer: Address,
}

pub struct StoreActorUpdated {
    pub old_store_actor: Address,
    pub new_store_actor: Address,
}

macro_rules! impl_noop_emit { ($($name:ident),* $(,)?) => { $(impl $name { #[inline(always)] pub fn emit_log(self) -> Result<(), ProgramError> { Ok(()) } })* }; }
impl_noop_emit!(
    StoreInitialized,
    TreasuryUpdated,
    PlatformFeeUpdated,
    DefaultReferralUpdated,
    MaxReferralUpdated,
    AuthorizedProgramAdded,
    AuthorizedProgramUpdated,
    PaymentTokenAdded,
    PaymentTokenUpdated,
    GameStoreConfigInitialized,
    GameStoreActiveUpdated,
    GamePaymentOptionSet,
    GamePaymentOptionRemoved,
    DiscountSet,
    DiscountCleared,
    ReferralBpsUpdated,
    GamePurchased,
    PurchaseReceiptCreated,
    StoreActorUpdated
);
