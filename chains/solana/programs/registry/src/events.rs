use anchor_lang::prelude::*;

#[event]
pub struct GameRegistered {
    pub game_id: String,
    pub publisher: Pubkey,
    pub pgc_program: Pubkey,
    pub pgc_game: Pubkey,
}

#[event]
pub struct GameStatusChanged {
    pub game_id: String,
    pub active: bool,
}
