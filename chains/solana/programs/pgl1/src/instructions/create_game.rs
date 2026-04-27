use quasar_lang::{cpi, prelude::*, sysvars::Sysvar};

use crate::{
    errors::PglError,
    events::GameCreated,
    state::{
        CreatorState, Game, PglConfig, CREATOR_STATE_SEED, GAME_SEED, MAX_GAME_ID_LEN,
        MAX_METADATA_URI_LEN, PGL_CONFIG_SEED,
    },
};

#[derive(Accounts)]
pub struct CreateGame<'info> {
    pub creator: &'info mut Signer,
    #[account(seeds = [PGL_CONFIG_SEED], bump = pgl_config.bump)]
    pub pgl_config: &'info Account<PglConfig>,
    #[account(mut, address = pgl_config.treasury @ PglError::Unauthorized)]
    pub treasury: &'info UncheckedAccount,
    #[account(
        init_if_needed,
        payer = creator,
        space = <CreatorState as Space>::SPACE,
        seeds = [CREATOR_STATE_SEED, creator],
        bump
    )]
    pub creator_state: &'info mut Account<CreatorState>,
    pub game: &'info mut UncheckedAccount,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(ctx: &mut Ctx<'info, CreateGame<'info>>) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let game_id = crate::instructions::read_string(ctx.data, &mut offset, MAX_GAME_ID_LEN)?;
    let metadata_uri =
        crate::instructions::read_string(ctx.data, &mut offset, MAX_METADATA_URI_LEN)?;

    require!(
        !game_id.trim().is_empty() && game_id.len() <= MAX_GAME_ID_LEN,
        PglError::InvalidGameId
    );
    require!(
        !metadata_uri.trim().is_empty() && metadata_uri.len() <= MAX_METADATA_URI_LEN,
        PglError::InvalidMetadataUri
    );

    let fee = ctx.accounts.pgl_config.create_game_fee_lamports.get();
    if fee > 0 {
        require!(
            ctx.accounts.creator.to_account_view().lamports() >= fee,
            PglError::InsufficientCreateGameFee
        );
        cpi::system::transfer(
            ctx.accounts.creator.to_account_view(),
            ctx.accounts.treasury.to_account_view(),
            fee,
        )
        .invoke()?;
    }

    let nonce = ctx.accounts.creator_state.next_nonce.get();
    let now = Clock::get()?.unix_timestamp.get();

    if ctx.accounts.creator_state.creator == Address::default() {
        ctx.accounts.creator_state.creator = *ctx.accounts.creator.address();
        ctx.accounts.creator_state.bump = ctx.bumps.creator_state;
    } else {
        require_keys_eq!(
            ctx.accounts.creator_state.creator,
            *ctx.accounts.creator.address(),
            PglError::Unauthorized
        );
    }

    let nonce_bytes = nonce.to_le_bytes();
    let (expected_game, bump) = quasar_lang::pda::based_try_find_program_address(
        &[
            GAME_SEED,
            ctx.accounts.creator.address().as_ref(),
            &nonce_bytes,
        ],
        &crate::ID,
    )?;
    require_keys_eq!(
        *ctx.accounts.game.address(),
        expected_game,
        PglError::GameAlreadyExists
    );
    require!(
        ctx.accounts.game.to_account_view().data_len() == 0,
        PglError::GameAlreadyExists
    );

    let bump_bytes = [bump];
    let seeds = [
        cpi::Seed::from(GAME_SEED),
        cpi::Seed::from(ctx.accounts.creator.address().as_ref()),
        cpi::Seed::from(&nonce_bytes),
        cpi::Seed::from(&bump_bytes),
    ];
    ctx.accounts
        .system_program
        .create_account_with_minimum_balance(
            ctx.accounts.creator,
            ctx.accounts.game,
            Game::SPACE as u64,
            &crate::ID,
            None,
        )?
        .invoke_signed(&seeds)?;

    let game_view =
        unsafe { &mut *(ctx.accounts.game as *mut UncheckedAccount as *mut AccountView) };
    unsafe {
        core::ptr::copy_nonoverlapping(
            <Game as Discriminator>::DISCRIMINATOR.as_ptr(),
            game_view.data_mut_ptr(),
            <Game as Discriminator>::DISCRIMINATOR.len(),
        );
    }
    let mut game = Game::from_account_view(game_view)?;
    game.set_inner(
        *ctx.accounts.creator.address(),
        nonce,
        *ctx.accounts.creator.address(),
        now,
        bump,
        game_id,
        metadata_uri,
        ctx.accounts.creator.to_account_view(),
        None,
    )?;

    let next_nonce = nonce.checked_add(1).ok_or(PglError::NonceOverflow)?;
    ctx.accounts.creator_state.next_nonce = next_nonce.into();

    emit!(GameCreated {
        game: *ctx.accounts.game.address(),
        creator: *ctx.accounts.creator.address(),
        publisher: *ctx.accounts.creator.address(),
        nonce,
        game_id,
        metadata_uri,
        created_at: now,
    })?;

    Ok(())
}
