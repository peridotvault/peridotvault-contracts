use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::CreateGameFeeUpdated,
    state::{PglConfig, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct SetCreateGameFee<'info> {
    pub authority: &'info Signer,
    #[account(mut, seeds = [PGL_CONFIG_SEED], bump = pgl_config.bump)]
    pub pgl_config: &'info mut Account<PglConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetCreateGameFee<'info>>,
    create_game_fee_lamports: u64,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.pgl_config.authority,
        *ctx.accounts.authority.address(),
        PglError::Unauthorized
    );

    let old_fee = ctx.accounts.pgl_config.create_game_fee_lamports.get();
    ctx.accounts.pgl_config.create_game_fee_lamports = create_game_fee_lamports.into();

    emit!(CreateGameFeeUpdated {
        old_fee,
        new_fee: create_game_fee_lamports,
    })?;

    Ok(())
}
