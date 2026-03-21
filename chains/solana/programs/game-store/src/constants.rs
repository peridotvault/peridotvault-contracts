use anchor_lang::prelude::Pubkey;
use anchor_lang::system_program;

pub const STORE_STATE_SEED: &[u8] = b"game_store_state";

pub const MAX_GAME_ID_LEN: usize = 128;
pub const MAX_PRICE_CONFIGS: usize = 16;
pub const MAX_PUBLISHER_BALANCES: usize = 32;
pub const MAX_FEE_BPS: u16 = 10_000;

pub fn is_native_sol_payment_method(payment_method: &Pubkey) -> bool {
    *payment_method == system_program::ID
}
