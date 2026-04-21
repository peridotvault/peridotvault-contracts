use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::ReferralBpsUpdated,
    state::{AuthorizedSourceProgram, GameStoreConfig, SourceGameMirror, StoreConfig},
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
        seeds = [b"authorized_source_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedSourceProgram>,
    /// CHECK: trusted program id only
    pub source_program: UncheckedAccount<'info>,
    #[account(owner = source_program.key() @ StoreError::UnsupportedSourceGameOwner)]
    pub game: Account<'info, SourceGameMirror>,
    #[account(
        mut,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
}

pub fn handler(ctx: Context<SetReferralBps>, referral_bps: Option<u16>) -> Result<()> {
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
