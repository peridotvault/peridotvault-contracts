use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::LicenseRenewed,
    state::{
        AuthorizedActor, Game, License, AUTHORIZED_ACTOR_SEED, GAME_SEED, LICENSE_SEED,
    },
};

pub(crate) fn handler(ctx: Context<RenewLicense>, expires_at: i64) -> Result<()> {
    let authorized_actor = &ctx.accounts.authorized_actor;
    require!(authorized_actor.active, PglError::AuthorizedActorInactive);

    let now = Clock::get()?.unix_timestamp;
    require!(expires_at > now, PglError::InvalidExpiry);

    let license = &mut ctx.accounts.license;
    if let Some(current_exp) = license.expires_at {
        require!(expires_at > current_exp, PglError::InvalidExpiry);
    }
    license.expires_at = Some(expires_at);

    emit!(LicenseRenewed {
        license: license.key(),
        holder: license.holder,
        game: license.game,
        expires_at,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RenewLicense<'info> {
    pub actor: Signer<'info>,

    /// CHECK: target holder of the license.
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
