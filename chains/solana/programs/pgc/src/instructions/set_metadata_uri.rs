use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_2022_extensions::{
        spl_token_metadata_interface::state::Field, token_metadata_update_field,
        TokenMetadataUpdateField,
    },
};

use crate::{constants::*, errors::Pgc1Error, events::MetadataUriUpdated, states::GameState};

#[derive(Accounts)]
pub struct SetMetadataUri<'info> {
    pub publisher: Signer<'info>,

    #[account(
        mut,
        has_one = publisher @ Pgc1Error::Unauthorized
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: PDA authority
    #[account(
        seeds = [GAME_AUTHORITY_SEED, game_state.key().as_ref()],
        bump = game_state.authority_bump
    )]
    pub game_authority: UncheckedAccount<'info>,

    /// CHECK: token-2022 mint for this game
    #[account(mut, address = game_state.mint)]
    pub mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<SetMetadataUri>, metadata_uri: String) -> Result<()> {
    require!(!metadata_uri.trim().is_empty(), Pgc1Error::EmptyMetadataUri);
    require!(
        metadata_uri.len() <= MAX_METADATA_URI_LEN,
        Pgc1Error::StringTooLong
    );

    let game_state = &mut ctx.accounts.game_state;
    game_state.metadata_uri = metadata_uri.clone();

    let game_key = game_state.key();
    let authority_seeds: &[&[u8]] = &[
        GAME_AUTHORITY_SEED,
        game_key.as_ref(),
        &[game_state.authority_bump],
    ];
    let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

    token_metadata_update_field(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TokenMetadataUpdateField {
                program_id: ctx.accounts.token_program.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.game_authority.to_account_info(),
            },
            signer_seeds,
        ),
        Field::Uri,
        metadata_uri.clone(),
    )?;

    emit!(MetadataUriUpdated {
        game: game_state.key(),
        metadata_uri,
    });

    Ok(())
}
