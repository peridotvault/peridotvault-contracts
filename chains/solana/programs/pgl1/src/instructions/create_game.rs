use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    errors::PglError,
    events::GameCreated,
    state::{
        CreatorState, Game, PglConfig, CREATOR_STATE_SEED, GAME_SEED, MAX_GAME_ID_LEN,
        MAX_METADATA_URI_LEN, PGL_CONFIG_SEED,
    },
};

pub fn handler(ctx: Context<CreateGame>, game_id: String, metadata_uri: String) -> Result<()> {
    require!(
        !game_id.trim().is_empty() && game_id.len() <= MAX_GAME_ID_LEN,
        PglError::InvalidGameId
    );
    require!(
        !metadata_uri.trim().is_empty() && metadata_uri.len() <= MAX_METADATA_URI_LEN,
        PglError::InvalidMetadataUri
    );

    let config = &ctx.accounts.pgl_config;
    let creator = &ctx.accounts.creator;

    if config.create_game_fee_lamports > 0 {
        require!(
            creator.to_account_info().lamports() >= config.create_game_fee_lamports,
            PglError::InsufficientCreateGameFee
        );

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: creator.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
            },
        );
        transfer(cpi_ctx, config.create_game_fee_lamports)?;
    }

    let creator_state = &mut ctx.accounts.creator_state;
    let nonce = creator_state.next_nonce;
    let now = Clock::get()?.unix_timestamp;

    if creator_state.creator == Pubkey::default() {
        creator_state.creator = creator.key();
        creator_state.bump = ctx.bumps.creator_state;
    }

    let game = &mut ctx.accounts.game;
    game.creator = creator.key();
    game.nonce = nonce;
    game.publisher = creator.key();
    game.game_id = game_id.clone();
    game.metadata_uri = metadata_uri.clone();
    game.created_at = now;
    game.bump = ctx.bumps.game;

    creator_state.next_nonce = creator_state
        .next_nonce
        .checked_add(1)
        .ok_or(PglError::GameAlreadyExists)?;

    emit!(GameCreated {
        game: game.key(),
        creator: game.creator,
        publisher: game.publisher,
        nonce,
        game_id,
        metadata_uri,
        created_at: now,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: String, metadata_uri: String)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [PGL_CONFIG_SEED],
        bump = pgl_config.bump,
    )]
    pub pgl_config: Account<'info, PglConfig>,

    /// CHECK: treasury pubkey is validated against config.
    #[account(
        mut,
        address = pgl_config.treasury @ PglError::Unauthorized,
    )]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = creator,
        space = CreatorState::SPACE,
        seeds = [CREATOR_STATE_SEED, creator.key().as_ref()],
        bump,
    )]
    pub creator_state: Account<'info, CreatorState>,

    #[account(
        init,
        payer = creator,
        space = Game::SPACE,
        seeds = [GAME_SEED, creator.key().as_ref(), &creator_state.next_nonce.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,

    pub system_program: Program<'info, System>,
}
