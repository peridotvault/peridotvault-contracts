use anchor_lang::prelude::*;
use crate::TransferPublisher;

pub fn handler(
    ctx: Context<TransferPublisher>,
    new_publisher: Pubkey,
) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    game_account.publisher = new_publisher;

    Ok(())
}
