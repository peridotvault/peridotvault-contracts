use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::AuthorityUpdated,
    state::{PglConfig, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [PGL_CONFIG_SEED],
        bump = pgl_config.bump,
        has_one = authority @ PglError::Unauthorized
    )]
    pub pgl_config: Account<'info, PglConfig>,
}

pub(crate) fn handler(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
    require!(new_authority != Pubkey::default(), PglError::Unauthorized);

    let config = &mut ctx.accounts.pgl_config;
    let old_authority = config.authority;
    config.authority = new_authority;

    emit!(AuthorityUpdated {
        old_authority,
        new_authority,
    });

    Ok(())
}
