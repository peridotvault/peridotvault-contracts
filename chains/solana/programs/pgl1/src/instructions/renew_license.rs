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

    // H-2 (Access control, least privilege): the license holder MUST sign.
    // `holder: Signer` enforces this at the Anchor constraint level
    // (fail-closed before the handler runs). No delegated-renewal path:
    // actor-only renewal is rejected by construction.

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
    /// License's authorized actor record (active gate enforced in handler).
    /// CHECK: identity validated via authorized_actor PDA constraint; no
    /// signature required — renewal authority is the holder, not the actor.
    pub actor: UncheckedAccount<'info>,

    /// H-2 (Access control): license holder, must sign. Enforced by type.
    pub holder: Signer<'info>,

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
