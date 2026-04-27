use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::AuthorizedActorClosed,
    state::{AuthorizedActor, PglConfig, AUTHORIZED_ACTOR_SEED, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct CloseAuthorizedActor<'info> {
    pub authority: &'info mut Signer,
    pub actor: &'info UncheckedAccount,
    #[account(seeds = [PGL_CONFIG_SEED], bump = pgl_config.bump)]
    pub pgl_config: &'info Account<PglConfig>,
    #[account(mut, seeds = [AUTHORIZED_ACTOR_SEED, actor], bump = authorized_actor.bump)]
    pub authorized_actor: &'info mut Account<AuthorizedActor>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, CloseAuthorizedActor<'info>>,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.pgl_config.authority,
        *ctx.accounts.authority.address(),
        PglError::Unauthorized
    );
    require!(
        !ctx.accounts.authorized_actor.active.get(),
        PglError::AuthorizedActorStillActive
    );

    emit!(AuthorizedActorClosed {
        actor: ctx.accounts.authorized_actor.actor,
    })?;

    ctx.accounts
        .authorized_actor
        .close(ctx.accounts.authority.to_account_view())?;

    Ok(())
}
