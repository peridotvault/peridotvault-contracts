use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::AuthorizedActorDeactivated,
    state::{AuthorizedActor, PglConfig, AUTHORIZED_ACTOR_SEED, PGL_CONFIG_SEED},
};

pub(crate) fn handler(ctx: Context<DeactivateAuthorizedActor>) -> Result<()> {
    let config = &ctx.accounts.pgl_config;
    require_keys_eq!(config.authority, ctx.accounts.authority.key(), PglError::Unauthorized);

    let actor = &mut ctx.accounts.authorized_actor;
    actor.active = false;

    emit!(AuthorizedActorDeactivated { actor: actor.actor });

    Ok(())
}

#[derive(Accounts)]
pub struct DeactivateAuthorizedActor<'info> {
    pub authority: Signer<'info>,

    /// CHECK: this is the actor being deactivated and is only used as a seed value.
    pub actor: UncheckedAccount<'info>,

    #[account(
        seeds = [PGL_CONFIG_SEED],
        bump = pgl_config.bump,
    )]
    pub pgl_config: Account<'info, PglConfig>,

    #[account(
        mut,
        seeds = [AUTHORIZED_ACTOR_SEED, actor.key().as_ref()],
        bump = authorized_actor.bump,
    )]
    pub authorized_actor: Account<'info, AuthorizedActor>,
}
