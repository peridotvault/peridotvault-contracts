use anchor_lang::prelude::*;
use crate::RegisterGame;

pub fn handler(
    ctx: Context<RegisterGame>,
    game_id: String,
    pgc_program: Pubkey,
    pgc_game: Pubkey,
) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    game_account.game_id = game_id;
    game_account.publisher = ctx.accounts.publisher.key();
    game_account.pgc_program = pgc_program;
    game_account.pgc_game = pgc_game;
    game_account.active = true;
    game_account.created_at = Clock::get()?.unix_timestamp;
    game_account.bump = ctx.bumps.game_account;

    emit!(crate::events::GameRegistered {
        game_id: game_account.game_id.clone(),
        publisher: game_account.publisher,
        pgc_program,
        pgc_game,
    });

    Ok(())
}
