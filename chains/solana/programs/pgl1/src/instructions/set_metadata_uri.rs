use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::MetadataUriUpdated,
    state::{Game, MAX_METADATA_URI_LEN, GAME_SEED},
};

pub fn handler(ctx: Context<SetMetadataUri>, metadata_uri: String) -> Result<()> {
    require!(
        !metadata_uri.trim().is_empty() && metadata_uri.len() <= MAX_METADATA_URI_LEN,
        PglError::InvalidMetadataUri
    );

    let game = &mut ctx.accounts.game;
    let signer = ctx.accounts.publisher.key();
    require_keys_eq!(game.publisher, signer, PglError::Unauthorized);

    game.metadata_uri = metadata_uri.clone();

    emit!(MetadataUriUpdated {
        game: game.key(),
        publisher: game.publisher,
        metadata_uri,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetMetadataUri<'info> {
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_SEED, game.creator.as_ref(), &game.nonce.to_le_bytes()],
        bump = game.bump,
    )]
    pub game: Account<'info, Game>,
}
