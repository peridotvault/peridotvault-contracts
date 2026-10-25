use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::LicenseRenewed,
    state::{
        AuthorizedActor, Game, License, CURRENT_VERSION, AUTHORIZED_ACTOR_SEED, GAME_SEED,
        LICENSE_SEED,
    },
};

pub(crate) fn handler(ctx: Context<RenewLicense>, expires_at: i64) -> Result<()> {
    let authorized_actor = &ctx.accounts.authorized_actor;
    require!(authorized_actor.active, PglError::AuthorizedActorInactive);

    // M1 (Access control): at least one of holder / actor must sign.
    // Anchor cannot express OR-of-signers via account types (two `Signer`s
    // would require AND), so both stay `UncheckedAccount` with an explicit
    // checks-effects-interactions-friendly check up front.
    require!(
        ctx.accounts.holder.is_signer || ctx.accounts.actor.is_signer,
        PglError::MissingRenewalSigner
    );

    // M5 version gates (fail-closed; never misinterpret newer schemas).
    require!(
        ctx.accounts.game.version <= CURRENT_VERSION,
        PglError::UnsupportedVersion
    );
    require!(
        ctx.accounts.license.version <= CURRENT_VERSION,
        PglError::UnsupportedVersion
    );

    let now = Clock::get()?.unix_timestamp;
    require!(expires_at > now, PglError::InvalidExpiry);

    let license = &mut ctx.accounts.license;
    if let Some(current_exp) = license.expires_at {
        require!(expires_at > current_exp, PglError::InvalidExpiry);
    }

    let old_expires_at = license.expires_at;
    license.expires_at = Some(expires_at);

    emit!(LicenseRenewed {
        license: license.key(),
        actor: ctx.accounts.actor.key(),
        holder: license.holder,
        game: license.game,
        old_expires_at,
        new_expires_at: expires_at,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RenewLicense<'info> {
    /// M1: either `actor` or `holder` must sign (OR enforced in handler).
    /// CHECK: validated via authorized_actor constraint (actor == actor.key) and active flag.
    pub actor: UncheckedAccount<'info>,

    /// M1: either `holder` or `actor` must sign (OR enforced in handler).
    /// CHECK: target holder of the license; signer status checked in handler.
    pub holder: UncheckedAccount<'info>,

    #[account(
        seeds = [AUTHORIZED_ACTOR_SEED, actor.key().as_ref()],
        bump = authorized_actor.bump,
        constraint = authorized_actor.actor == actor.key() @ PglError::Unauthorized,
    )]
    pub authorized_actor: Account<'info, AuthorizedActor>,

    #[account(
        seeds = [GAME_SEED, game.creator.as_ref(), &game.nonce.to_le_bytes()],
        bump = game.bump,
    )]
    pub game: Account<'info, Game>,

    #[account(
        mut,
        seeds = [LICENSE_SEED, holder.key().as_ref(), game.key().as_ref()],
        bump = license.bump,
        constraint = license.holder == holder.key() @ PglError::Unauthorized,
        constraint = license.game == game.key() @ PglError::Unauthorized,
    )]
    pub license: Account<'info, License>,
}
