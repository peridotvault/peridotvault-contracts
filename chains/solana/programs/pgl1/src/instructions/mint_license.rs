use quasar_lang::{prelude::*, sysvars::Sysvar};

use crate::{
    errors::PglError,
    events::LicenseMinted,
    instructions::read_option_i64,
    state::{AuthorizedActor, Game, License, AUTHORIZED_ACTOR_SEED, GAME_SEED, LICENSE_SEED},
};

#[derive(Accounts)]
pub struct MintLicense<'info> {
    pub actor: &'info mut Signer,
    pub holder: &'info UncheckedAccount,
    #[account(seeds = [AUTHORIZED_ACTOR_SEED, actor], bump = authorized_actor.bump)]
    pub authorized_actor: &'info Account<AuthorizedActor>,
    pub game: Account<Game<'info>>,
    #[account(
        init,
        payer = actor,
        space = <License as Space>::SPACE,
        seeds = [LICENSE_SEED, holder, game],
        bump
    )]
    pub license: &'info mut Account<License>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(ctx: &mut Ctx<'info, MintLicense<'info>>) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let expires_at = read_option_i64(ctx.data, &mut offset)?;

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

    let now = Clock::get()?.unix_timestamp.get();
    if let Some(exp) = expires_at {
        require!(exp > now, PglError::InvalidExpiry);
    }

    require!(
        ctx.accounts.license.holder == Address::default()
            && ctx.accounts.license.game == Address::default(),
        PglError::LicenseAlreadyExists
    );

    ctx.accounts.license.set_inner(
        *ctx.accounts.holder.address(),
        *ctx.accounts.game.address(),
        now,
        expires_at.into(),
        ctx.bumps.license,
    );

    emit!(LicenseMinted {
        license: *ctx.accounts.license.address(),
        holder: *ctx.accounts.holder.address(),
        game: *ctx.accounts.game.address(),
        issued_at: now,
        expires_at,
    })?;

    Ok(())
}
