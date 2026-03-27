use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PgcError;
use crate::events::GameCreated;
use crate::CreateGame;

pub fn handler(ctx: Context<CreateGame>, game_id: String, metadata_uri: String, mint: Option<Pubkey>) -> Result<()> {
    require!(game_id.len() <= MAX_GAME_ID_LEN, PgcError::InvalidGameId);
    require!(metadata_uri.len() <= MAX_METADATA_URI_LEN, PgcError::InvalidMetadataUri);

    let game_account = &mut ctx.accounts.game_account;
    game_account.game_id = game_id.clone();
    game_account.publisher = ctx.accounts.publisher.key();
    game_account.metadata_uri = metadata_uri.clone();
    game_account.mint = mint;
    game_account.bump = ctx.bumps.game_account;

    emit!(GameCreated {
        game_id,
        publisher: game_account.publisher,
        metadata_uri,
    });

    Ok(())
}
