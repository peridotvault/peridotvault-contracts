use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::AuthorizedActorDeactivated,
    state::{AuthorizedActor, PglConfig, AUTHORIZED_ACTOR_SEED, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct DeactivateAuthorizedActor<'info> {
    pub authority: &'info Signer,
    pub actor: &'info UncheckedAccount,
    #[account(seeds = [PGL_CONFIG_SEED], bump = pgl_config.bump)]
    pub pgl_config: &'info Account<PglConfig>,
    #[account(mut, seeds = [AUTHORIZED_ACTOR_SEED, actor], bump = authorized_actor.bump)]
    pub authorized_actor: &'info mut Account<AuthorizedActor>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, DeactivateAuthorizedActor<'info>>,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.pgl_config.authority,
        *ctx.accounts.authority.address(),
        PglError::Unauthorized
    );

    ctx.accounts.authorized_actor.active = false.into();

    emit!(AuthorizedActorDeactivated {
        actor: ctx.accounts.authorized_actor.actor,
    })?;

    Ok(())
}
