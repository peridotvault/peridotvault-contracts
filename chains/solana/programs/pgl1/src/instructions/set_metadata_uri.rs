use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::MetadataUriUpdated,
    state::{Game, GAME_SEED, MAX_METADATA_URI_LEN},
};

#[derive(Accounts)]
pub struct SetMetadataUri<'info> {
    pub publisher: &'info Signer,
    #[account(mut)]
    pub game: Account<Game<'info>>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetMetadataUri<'info>>,
) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let metadata_uri =
        crate::instructions::read_string(ctx.data, &mut offset, MAX_METADATA_URI_LEN)?;
    require!(
        !metadata_uri.trim().is_empty() && metadata_uri.len() <= MAX_METADATA_URI_LEN,
        PglError::InvalidMetadataUri
    );
    require_keys_eq!(
        ctx.accounts.game.publisher,
        *ctx.accounts.publisher.address(),
        PglError::Unauthorized
    );

    let nonce_bytes = ctx.accounts.game.nonce.get().to_le_bytes();
    quasar_lang::pda::verify_program_address(
        &[GAME_SEED, ctx.accounts.game.creator.as_ref(), &nonce_bytes],
        &crate::ID,
        ctx.accounts.game.address(),
    )?;

    let old_uri = ctx.accounts.game.metadata_uri();
    emit!(MetadataUriUpdated {
        game: *ctx.accounts.game.address(),
        publisher: ctx.accounts.game.publisher,
        old_uri,
        new_uri: metadata_uri,
    })?;

    ctx.accounts
        .game
        .set_metadata_uri(ctx.accounts.publisher, metadata_uri)?;

    Ok(())
}
