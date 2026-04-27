use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::CreatorStateClosed,
    state::{CreatorState, CREATOR_STATE_SEED},
};

#[derive(Accounts)]
pub struct CloseCreatorState<'info> {
    pub creator: &'info mut Signer,
    #[account(mut, seeds = [CREATOR_STATE_SEED, creator], bump = creator_state.bump)]
    pub creator_state: &'info mut Account<CreatorState>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, CloseCreatorState<'info>>,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.creator_state.creator,
        *ctx.accounts.creator.address(),
        PglError::Unauthorized
    );
    require!(
        ctx.accounts.creator_state.next_nonce.get() == 0,
        PglError::CreatorStateNotEmpty
    );

    emit!(CreatorStateClosed {
        creator: ctx.accounts.creator_state.creator,
    })?;

    ctx.accounts
        .creator_state
        .close(ctx.accounts.creator.to_account_view())?;

    Ok(())
}
