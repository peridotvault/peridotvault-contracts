use anchor_lang::prelude::*;
use crate::Initialize;

pub fn handler(
    ctx: Context<Initialize>,
    authority: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.registry_config;
    config.authority = authority;
    config.bump = ctx.bumps.registry_config;

    Ok(())
}
