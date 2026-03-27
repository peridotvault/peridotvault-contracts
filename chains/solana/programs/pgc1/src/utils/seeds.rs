use anchor_lang::prelude::*;
use crate::constants::*;

pub fn get_game_pda(game_id: &str, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_GAME, game_id.as_bytes()],
        program_id
    )
}

pub fn get_license_pda(owner: &Pubkey, game: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_LICENSE, owner.as_ref(), game.as_ref()],
        program_id
    )
}
