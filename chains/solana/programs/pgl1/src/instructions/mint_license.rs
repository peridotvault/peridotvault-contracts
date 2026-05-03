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
    // Defensive guard: reject duplicate mint if this PDA was already initialized before.
    require!(
        license.holder == Pubkey::default() && license.game == Pubkey::default(),
        PglError::LicenseAlreadyExists
    );
    license.holder = ctx.accounts.holder.key();
    license.game = ctx.accounts.game.key();
    license.issued_at = now;
    license.expires_at = expires_at;
    license.bump = ctx.bumps.license;

    emit!(LicenseMinted {
        license: license.key(),
        actor: ctx.accounts.actor.key(),
        holder: license.holder,
        game: license.game,
        issued_at: now,
        expires_at,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MintLicense<'info> {
    /// CHECK: validated via authorized_actor constraint.
    pub actor: UncheckedAccount<'info>,

    /// Holder who receives the license and pays rent.
    #[account(mut)]
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
        init,
        payer = holder,
        space = License::SPACE,
        seeds = [LICENSE_SEED, holder.key().as_ref(), game.key().as_ref()],
        bump,
    )]
    pub license: Account<'info, License>,

    pub system_program: Program<'info, System>,
}
