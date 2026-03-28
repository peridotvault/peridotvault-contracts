use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<UpdateMetadataUri>, new_uri: String) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    game_account.metadata_uri = new_uri;
    Ok(())
}

