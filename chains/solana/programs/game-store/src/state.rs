use quasar_lang::prelude::*;

pub const ROLE_SOURCE: u8 = 0;
pub const ROLE_REGISTRY: u8 = 1;

pub const BPS_DENOMINATOR: u64 = 10_000;
pub const PLATFORM_FEE_BPS_MAX: u16 = 10_000;
pub const MAX_REFERRAL_BPS_HARD_CAP: u16 = 5_000;

#[repr(C)]
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct OptionU16 {
    tag: u8,
    value: [u8; 2],
}

impl OptionU16 {
    pub const NONE: Self = Self {
        tag: 0,
        value: [0; 2],
    };

    #[inline(always)]
    pub fn get(&self) -> Option<u16> {
        if self.tag == 0 {
            None
        } else {
            Some(u16::from_le_bytes(self.value))
        }
    }

    #[inline(always)]
    pub fn set(&mut self, value: Option<u16>) {
        match value {
            None => *self = Self::NONE,
            Some(v) => {
                self.tag = 1;
                self.value = v.to_le_bytes();
            }
        }
    }
}

impl From<Option<u16>> for OptionU16 {
    #[inline(always)]
    fn from(value: Option<u16>) -> Self {
        let mut out = Self::NONE;
        out.set(value);
        out
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct OptionI64 {
    tag: u8,
    value: [u8; 8],
}

impl OptionI64 {
    pub const NONE: Self = Self {
        tag: 0,
        value: [0; 8],
    };

    #[inline(always)]
    pub fn get(&self) -> Option<i64> {
        if self.tag == 0 {
            None
        } else {
            Some(i64::from_le_bytes(self.value))
        }
    }

    #[inline(always)]
    pub fn set(&mut self, value: Option<i64>) {
        match value {
            None => *self = Self::NONE,
            Some(v) => {
                self.tag = 1;
                self.value = v.to_le_bytes();
            }
        }
    }
}

impl From<Option<i64>> for OptionI64 {
    #[inline(always)]
    fn from(value: Option<i64>) -> Self {
        let mut out = Self::NONE;
        out.set(value);
        out
    }
}

const _: () = assert!(core::mem::align_of::<OptionU16>() == 1);
const _: () = assert!(core::mem::size_of::<OptionU16>() == 3);
const _: () = assert!(core::mem::align_of::<OptionI64>() == 1);
const _: () = assert!(core::mem::size_of::<OptionI64>() == 9);

#[account(discriminator = [108, 23, 66, 65, 67, 124, 167, 135])]
pub struct StoreConfig {
    pub authority: Address,
    pub treasury: Address,
    pub platform_fee_bps: u16,
    pub default_referral_bps: u16,
    pub max_referral_bps: u16,
    pub store_actor: Address,
    pub bump: u8,
}

#[account(discriminator = [18, 164, 77, 11, 61, 253, 148, 223])]
pub struct AuthorizedProgram {
    pub program_id: Address,
    pub active: bool,
    pub role: u8,
    pub bump: u8,
}

#[account(discriminator = [101, 168, 82, 98, 20, 218, 130, 107])]
pub struct AcceptedPaymentToken {
    pub mint: Address,
    pub active: bool,
    pub bump: u8,
}

#[account(discriminator = [147, 51, 220, 95, 81, 151, 19, 208])]
pub struct GameStoreConfig {
    pub game: Address,
    pub active: bool,
    pub referral_bps: OptionU16,
    pub discount_bps: OptionU16,
    pub discount_starts_at: OptionI64,
    pub discount_expires_at: OptionI64,
    pub bump: u8,
}

#[account(discriminator = [7, 115, 189, 102, 10, 48, 220, 137])]
pub struct GamePaymentOption {
    pub game: Address,
    pub mint: Address,
    pub base_price: u64,
    pub active: bool,
    pub bump: u8,
}

#[account(discriminator = [79, 127, 222, 137, 154, 131, 150, 134])]
pub struct PurchaseReceipt {
    pub buyer: Address,
    pub game: Address,
    pub payment_mint: Address,
    pub paid_amount: u64,
    pub final_price: u64,
    pub referrer: Address,
    pub referral_bps_applied: u16,
    pub purchased_at: i64,
    pub bump: u8,
}
