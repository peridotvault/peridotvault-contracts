use quasar_lang::{prelude::*, sysvars::Sysvar};

use crate::{
    errors::PglError,
    events::LicenseRenewed,
    state::{AuthorizedActor, Game, License, AUTHORIZED_ACTOR_SEED, GAME_SEED, LICENSE_SEED},
};

#[derive(Accounts)]
pub struct RenewLicense<'info> {
    pub actor: &'info Signer,
    pub holder: &'info UncheckedAccount,
    #[account(seeds = [AUTHORIZED_ACTOR_SEED, actor], bump = authorized_actor.bump)]
    pub authorized_actor: &'info Account<AuthorizedActor>,
    pub game: Account<Game<'info>>,
    #[account(mut, seeds = [LICENSE_SEED, holder, game], bump = license.bump)]
    pub license: &'info mut Account<License>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, RenewLicense<'info>>,
    expires_at: i64,
) -> Result<(), ProgramError> {
    require!(
        ctx.accounts.authorized_actor.active.get(),
        PglError::AuthorizedActorInactive
    );
    require_keys_eq!(
        ctx.accounts.authorized_actor.actor,
        *ctx.accounts.actor.address(),
        PglError::Unauthorized
    );

    let nonce_bytes = ctx.accounts.game.nonce.get().to_le_bytes();
    quasar_lang::pda::verify_program_address(
        &[GAME_SEED, ctx.accounts.game.creator.as_ref(), &nonce_bytes],
        &crate::ID,
        ctx.accounts.game.address(),
    )?;
    require_keys_eq!(
        ctx.accounts.license.holder,
        *ctx.accounts.holder.address(),
        PglError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.license.game,
        *ctx.accounts.game.address(),
        PglError::Unauthorized
    );

    let now = Clock::get()?.unix_timestamp.get();
    require!(expires_at > now, PglError::InvalidExpiry);

    let old_expires_at = ctx.accounts.license.expires_at.get();
    if let Some(current_exp) = old_expires_at {
        require!(expires_at > current_exp, PglError::InvalidExpiry);
    }

    ctx.accounts.license.expires_at = Some(expires_at).into();

    emit!(LicenseRenewed {
        license: *ctx.accounts.license.address(),
        holder: ctx.accounts.license.holder,
        game: ctx.accounts.license.game,
        old_expires_at,
        new_expires_at: expires_at,
    })?;

    Ok(())
}
