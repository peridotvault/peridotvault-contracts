use quasar_lang::prelude::*;

pub const PGL_CONFIG_SEED: &[u8] = b"pgl_config";
pub const AUTHORIZED_ACTOR_SEED: &[u8] = b"authorized_actor";
pub const CREATOR_STATE_SEED: &[u8] = b"creator_state";
pub const GAME_SEED: &[u8] = b"game";
pub const LICENSE_SEED: &[u8] = b"license";

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
    #[inline(always)]
    fn from(value: Option<i64>) -> Self {
        let mut out = Self::NONE;
        out.set(value);
        out
    }
}

const _: () = assert!(core::mem::align_of::<OptionI64>() == 1);
const _: () = assert!(core::mem::size_of::<OptionI64>() == 9);

#[account(discriminator = [152, 183, 211, 24, 96, 186, 93, 22])]
pub struct PglConfig {
    pub authority: Address,
    pub treasury: Address,
    pub create_game_fee_lamports: u64,
    pub bump: u8,
}

#[account(discriminator = [155, 89, 1, 231, 51, 170, 32, 23])]
pub struct AuthorizedActor {
    pub actor: Address,
    pub active: bool,
    pub bump: u8,
}

#[account(discriminator = [37, 107, 190, 213, 241, 216, 73, 180])]
pub struct CreatorState {
    pub creator: Address,
    pub next_nonce: u64,
    pub bump: u8,
}

#[account(discriminator = [27, 90, 166, 125, 74, 100, 121, 18])]
pub struct Game<'info> {
    pub creator: Address,
    pub nonce: u64,
    pub publisher: Address,
    pub created_at: i64,
    pub bump: u8,
    pub game_id: String<u32, 64>,
    pub metadata_uri: String<u32, 256>,
}

#[account(discriminator = [248, 152, 195, 100, 185, 108, 176, 231])]
pub struct License {
    pub holder: Address,
    pub game: Address,
    pub issued_at: i64,
    pub expires_at: OptionI64,
    pub bump: u8,
}
