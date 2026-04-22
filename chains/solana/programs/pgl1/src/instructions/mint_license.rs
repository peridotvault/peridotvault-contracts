use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::LicenseMinted,
    state::{
        AuthorizedActor, Game, License, AUTHORIZED_ACTOR_SEED, GAME_SEED, LICENSE_SEED,
    },
};

pub(crate) fn handler(ctx: Context<MintLicense>, expires_at: Option<i64>) -> Result<()> {
    let authorized_actor = &ctx.accounts.authorized_actor;
    require!(authorized_actor.active, PglError::AuthorizedActorInactive);

    let now = Clock::get()?.unix_timestamp;
    if let Some(exp) = expires_at {
        require!(exp > now, PglError::InvalidExpiry);
    }

    let license = &mut ctx.accounts.license;
    license.holder = ctx.accounts.holder.key();
    license.game = ctx.accounts.game.key();
    license.issued_at = now;
    license.expires_at = expires_at;
    license.bump = ctx.bumps.license;

    emit!(LicenseMinted {
        license: license.key(),
        holder: license.holder,
        game: license.game,
        issued_at: now,
        expires_at,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MintLicense<'info> {
    #[account(mut)]
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
        init,
        payer = actor,
        space = License::SPACE,
        seeds = [LICENSE_SEED, holder.key().as_ref(), game.key().as_ref()],
        bump,
    )]
    pub license: Account<'info, License>,

    pub system_program: Program<'info, System>,
}
