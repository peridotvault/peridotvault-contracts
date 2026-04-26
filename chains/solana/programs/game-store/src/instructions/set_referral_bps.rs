use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::ReferralBpsUpdated,
    state::{AuthorizedProgram, GameStoreConfig, StoreConfig},
};

#[derive(Accounts)]
pub struct SetReferralBps<'info> {
    pub publisher: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
    )]
    pub store_config: Account<'info, StoreConfig>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedProgram>,
    pub source_program: Program<'info, pgl1::program::Pgl1>,
    pub game: Account<'info, pgl1::state::Game>,
    #[account(
        mut,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
}

pub(crate) fn handler(ctx: Context<SetReferralBps>, referral_bps: Option<u16>) -> Result<()> {
    require_keys_eq!(ctx.accounts.game.publisher, ctx.accounts.publisher.key(), StoreError::Unauthorized);

    let normalized = match referral_bps {
        None => None,
        Some(0) => None,
        Some(v) => {
            require!(v <= ctx.accounts.store_config.max_referral_bps, StoreError::ReferralAboveMax);
            Some(v)
        }
    };

    ctx.accounts.game_store_config.referral_bps = normalized;
    emit!(ReferralBpsUpdated {
        game: ctx.accounts.game.key(),
        referral_bps: normalized,
    });
    Ok(())
}
