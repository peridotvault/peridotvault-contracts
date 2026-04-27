use quasar_lang::prelude::*;

pub const REGISTRY_CONFIG_SEED: &[u8] = b"registry_config";
pub const ACCEPTED_PAYMENT_TOKEN_SEED: &[u8] = b"accepted_payment_token";
pub const REGISTRY_GAME_SEED: &[u8] = b"registry_game";
pub const PUBLISH_GRANT_SEED: &[u8] = b"publish_grant";

pub const MAX_GAME_ID_LEN: usize = 64;
pub const MAX_METADATA_URI_LEN: usize = 256;

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
    fn from(value: Option<i64>) -> Self {
        let mut out = Self::NONE;
        out.set(value);
        out
    }
}

const _: () = assert!(core::mem::align_of::<OptionI64>() == 1);
const _: () = assert!(core::mem::size_of::<OptionI64>() == 9);

#[repr(C)]
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct GameStatusPod(pub u8);

impl GameStatusPod {
    pub fn get(&self) -> GameStatus {
        GameStatus::from_u8(self.0).unwrap_or(GameStatus::Banned)
    }
}

impl From<GameStatus> for GameStatusPod {
    fn from(value: GameStatus) -> Self {
        Self(value.as_u8())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameStatus {
    Active,
    Suspended,
    Banned,
}

impl GameStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Suspended),
            2 => Some(Self::Banned),
            _ => None,
        }
    }
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Active => 0,
            Self::Suspended => 1,
            Self::Banned => 2,
        }
    }
}

#[account(discriminator = [23, 118, 10, 246, 173, 231, 243, 156])]
pub struct RegistryConfig {
    pub authority: Address,
    pub treasury: Address,
    pub pgl1_program: Address,
    pub bump: u8,
}

#[account(discriminator = [101, 168, 82, 98, 20, 218, 130, 107])]
pub struct AcceptedPaymentToken {
    pub mint: Address,
    pub active: bool,
    pub fee_amount: u64,
    pub bump: u8,
}

#[account(discriminator = [44, 59, 51, 135, 203, 140, 48, 151])]
pub struct RegistryGame<'info> {
    pub game: Address,
    pub registered_at: i64,
    pub status: GameStatusPod,
    pub bump: u8,
    pub game_id: String<u32, 64>,
}

#[account(discriminator = [96, 84, 122, 50, 0, 193, 171, 194])]
pub struct PublishGrant {
    pub expired_at: OptionI64,
    pub bump: u8,
}

impl PublishGrant {
    pub fn is_active_at(&self, now: i64) -> bool {
        match self.expired_at.get() {
            None => true,
            Some(ts) => now <= ts,
        }
    }
}
