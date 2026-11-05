use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::LicenseRevoked,
    state::{
        AuthorizedActor, Game, License, CURRENT_VERSION, AUTHORIZED_ACTOR_SEED,
        GAME_SEED, LICENSE_SEED,
    },
};

/// ANT-941 FR4-ON: burn/lock path for onchain licenses.
///
/// Seizure-style revoke: the authorized actor (publisher/store operator)
/// closes the license PDA WITHOUT the holder's signature, so a malicious
/// holder cannot block revocation. Rent lamports go back to the holder
/// (their own rent). The indexer observes `LicenseRevoked` and flips the
/// mirror row to inactive (`POST /internal/onchain/sync status=revoked`).
///
/// Lenses:
/// - Access control: actor must sign AND map to an active authorized_actor
///   PDA (least privilege; holders/anyone else cannot revoke).
/// - Upgradeability safety: M5 version gates (fail-closed), no state layout
///   change (append-only instruction).
/// - Checks-effects-interactions: explicit holder/game binding checks, then
///   emit; Anchor settles `close` at exit (rent -> holder).
pub(crate) fn handler(ctx: Context<RevokeLicense>) -> Result<()> {
    let authorized_actor = &ctx.accounts.authorized_actor;
    require!(authorized_actor.active, PglError::AuthorizedActorInactive);

    // M5 version gates (fail-closed; never misinterpret newer schemas).
    require!(
        ctx.accounts.game.version <= CURRENT_VERSION,
        PglError::UnsupportedVersion
    );
    require!(
        ctx.accounts.license.version <= CURRENT_VERSION,
        PglError::UnsupportedVersion
    );

    // Defense in depth: seeds already bind holder+game, but `holder` is an
    // UncheckedAccount (no signature), so verify the binding explicitly.
    require_keys_eq!(
        ctx.accounts.license.holder,
        ctx.accounts.holder.key(),
        PglError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.license.game,
        ctx.accounts.game.key(),
        PglError::Unauthorized
    );

    let now = Clock::get()?.unix_timestamp;

    emit!(LicenseRevoked {
        license: ctx.accounts.license.key(),
        actor: ctx.accounts.actor.key(),
        holder: ctx.accounts.holder.key(),
        game: ctx.accounts.game.key(),
        revoked_at: now,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RevokeLicense<'info> {
    /// H-1 (Access control): authorized revoke actor, must sign.
    /// Identity validated via authorized_actor PDA constraint below.
    pub actor: Signer<'info>,

    /// CHECK: license holder; receives the closed account's rent.
    /// Must NOT sign (seizure path). Binding verified against the
    /// license account in the handler.
    #[account(mut)]
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
        close = holder,
        seeds = [LICENSE_SEED, holder.key().as_ref(), game.key().as_ref()],
        bump = license.bump,
    )]
    pub license: Account<'info, License>,
}
