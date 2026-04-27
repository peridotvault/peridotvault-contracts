use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::ReferralBpsUpdated,
    external::{Pgl1Program, PglGame},
    instructions::read_option_u16,
    state::{AuthorizedProgram, GameStoreConfig, StoreConfig},
};

#[derive(Accounts)]
pub struct SetReferralBps<'info> {
    pub publisher: &'info Signer,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
    )]
    pub store_config: &'info Account<StoreConfig>,
    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,
    pub source_program: &'info Program<Pgl1Program>,
    pub game: &'info Account<PglGame>,
    #[account(
        mut,
        seeds = [b"game_store_config", game],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: &'info mut Account<GameStoreConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetReferralBps<'info>>,
) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let referral_bps = read_option_u16(ctx.data, &mut offset)?;
    require_keys_eq!(
        ctx.accounts.game.publisher()?,
        *ctx.accounts.publisher.address(),
        StoreError::Unauthorized
    );

    let normalized = match referral_bps {
        None => None,
        Some(0) => None,
        Some(v) => {
            require!(
                v <= ctx.accounts.store_config.max_referral_bps.get(),
                StoreError::ReferralAboveMax
            );
            Some(v)
        }
    };

    ctx.accounts.game_store_config.referral_bps.set(normalized);
    emit!(ReferralBpsUpdated {
        game: *ctx.accounts.game.address(),
        referral_bps_present: normalized.is_some(),
        referral_bps: normalized.unwrap_or(0),
    })?;
    Ok(())
}
