use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::CreateGameFeeUpdated,
    state::{PglConfig, PGL_CONFIG_SEED},
};

pub fn handler(ctx: Context<SetCreateGameFee>, create_game_fee_lamports: u64) -> Result<()> {
    let config = &mut ctx.accounts.pgl_config;
    require_keys_eq!(config.authority, ctx.accounts.authority.key(), PglError::Unauthorized);

    config.create_game_fee_lamports = create_game_fee_lamports;

    emit!(CreateGameFeeUpdated {
        authority: config.authority,
        create_game_fee_lamports,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetCreateGameFee<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [PGL_CONFIG_SEED],
        bump = pgl_config.bump,
    )]
    pub pgl_config: Account<'info, PglConfig>,
}
