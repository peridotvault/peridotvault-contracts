use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::CreatorStateClosed,
    state::{CreatorState, CREATOR_STATE_SEED},
};

pub(crate) fn handler(ctx: Context<CloseCreatorState>) -> Result<()> {
    let creator_state = &ctx.accounts.creator_state;
    require!(creator_state.next_nonce == 0, PglError::CreatorStateNotEmpty);

    emit!(CreatorStateClosed {
        creator: creator_state.creator,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CloseCreatorState<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        close = creator,
        seeds = [CREATOR_STATE_SEED, creator.key().as_ref()],
        bump = creator_state.bump,
        constraint = creator_state.creator == creator.key() @ PglError::Unauthorized,
    )]
    pub creator_state: Account<'info, CreatorState>,
}
