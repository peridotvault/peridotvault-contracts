use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::AuthorizedActorClosed,
    state::{AuthorizedActor, PglConfig, AUTHORIZED_ACTOR_SEED, PGL_CONFIG_SEED},
};

pub(crate) fn handler(ctx: Context<CloseAuthorizedActor>) -> Result<()> {
    let config = &ctx.accounts.pgl_config;
    require_keys_eq!(config.authority, ctx.accounts.authority.key(), PglError::Unauthorized);

    let actor = &ctx.accounts.authorized_actor;
    require!(!actor.active, PglError::AuthorizedActorStillActive);

    emit!(AuthorizedActorClosed { actor: actor.actor });

    Ok(())
}

#[derive(Accounts)]
pub struct CloseAuthorizedActor<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: this is the actor being closed and is only used as a seed value.
    pub actor: UncheckedAccount<'info>,

    #[account(
        seeds = [PGL_CONFIG_SEED],
        bump = pgl_config.bump,
    )]
    pub pgl_config: Account<'info, PglConfig>,

    #[account(
        mut,
        close = authority,
        seeds = [AUTHORIZED_ACTOR_SEED, actor.key().as_ref()],
        bump = authorized_actor.bump,
    )]
    pub authorized_actor: Account<'info, AuthorizedActor>,
}
