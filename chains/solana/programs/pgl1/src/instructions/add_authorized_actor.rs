use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::AuthorizedActorAdded,
    state::{AuthorizedActor, PglConfig, AUTHORIZED_ACTOR_SEED, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct AddAuthorizedActor<'info> {
    pub authority: &'info mut Signer,
    pub actor: &'info UncheckedAccount,
    #[account(seeds = [PGL_CONFIG_SEED], bump = pgl_config.bump)]
    pub pgl_config: &'info Account<PglConfig>,
    #[account(
        init,
        payer = authority,
        space = <AuthorizedActor as Space>::SPACE,
        seeds = [AUTHORIZED_ACTOR_SEED, actor],
        bump
    )]
    pub authorized_actor: &'info mut Account<AuthorizedActor>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, AddAuthorizedActor<'info>>,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.pgl_config.authority,
        *ctx.accounts.authority.address(),
        PglError::Unauthorized
    );

    ctx.accounts.authorized_actor.set_inner(
        *ctx.accounts.actor.address(),
        true,
        ctx.bumps.authorized_actor,
    );

    emit!(AuthorizedActorAdded {
        actor: *ctx.accounts.actor.address(),
    })?;

    Ok(())
}
