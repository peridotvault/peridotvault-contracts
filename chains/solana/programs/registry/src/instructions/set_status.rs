use anchor_lang::prelude::*;
use crate::SetStatus;

pub fn handler(
    ctx: Context<SetStatus>,
    active: bool,
) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    game_account.active = active;

    emit!(crate::events::GameStatusChanged {
        game_id: game_account.game_id.clone(),
        active,
    });

    Ok(())
}
