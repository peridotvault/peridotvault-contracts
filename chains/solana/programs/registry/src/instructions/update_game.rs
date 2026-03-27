use anchor_lang::prelude::*;
use crate::UpdateGame;

pub fn handler(
    ctx: Context<UpdateGame>,
    pgc_program: Pubkey,
    pgc_game: Pubkey,
) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    game_account.pgc_program = pgc_program;
    game_account.pgc_game = pgc_game;

    Ok(())
}
