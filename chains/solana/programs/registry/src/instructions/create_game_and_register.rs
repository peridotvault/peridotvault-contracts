use anchor_lang::{prelude::*, solana_program::instruction::Instruction, solana_program::program, solana_program::system_program};
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{
    errors::RegistryError,
    events::GameRegistered,
    state::{
        AcceptedPaymentToken, GameStatus, PublishGrant, RegistryConfig, RegistryGame, MAX_GAME_ID_LEN, MAX_METADATA_URI_LEN,
    },
};

pub const GAME_STORE_PROGRAM_ID: Pubkey = pubkey!("6gTd8TQ9NiC7yxBfGWBzH1aWdk77fg779nUJhYTrEsPd");

// CPI discriminators for game-store program.
// These are computed from Anchor's instruction name hashing:
//   sha256("anchor:init_game_store_config")[:8]
//   sha256("anchor:set_game_payment_option")[:8]
// SECURITY: If the game-store program is upgraded and instruction layout changes,
// these discriminators must be updated. Consider migrating to a proper CPI crate
// import when the circular dependency (registry <-> game-store) is resolved.
const INIT_GAME_STORE_CONFIG_DISC: [u8; 8] = [0x7e, 0xd2, 0xfe, 0x0b, 0x7c, 0x57, 0xe4, 0xa3];
const SET_GAME_PAYMENT_OPTION_DISC: [u8; 8] = [0x23, 0x98, 0x38, 0xe4, 0x80, 0xa1, 0xa2, 0xae];

#[derive(Accounts)]
pub struct CreateGameAndRegister<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump
    )]
    pub config: Account<'info, RegistryConfig>,

    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"accepted_payment_token", payment_mint.key().as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,

    #[account(
        mut,
        constraint = publisher_payment_account.owner == publisher.key() @ RegistryError::Unauthorized,
        constraint = publisher_payment_account.mint == payment_mint.key() @ RegistryError::PaymentTokenNotAllowed
    )]
    pub publisher_payment_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = treasury_payment_account.owner == config.treasury @ RegistryError::InvalidTreasury,
        constraint = treasury_payment_account.mint == payment_mint.key() @ RegistryError::PaymentTokenNotAllowed
    )]
    pub treasury_payment_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = publisher,
        space = RegistryGame::SPACE,
        seeds = [b"registry_game", game.key().as_ref()],
        bump
    )]
    pub registry_game: Account<'info, RegistryGame>,

    /// CHECK: expected to be created by CPI into the PGL-1 program.
    #[account(mut)]
    pub game: UncheckedAccount<'info>,

    /// CHECK: forwarded to PGL-1 create_game CPI.
    #[account(mut)]
    pub pgl_creator_state: UncheckedAccount<'info>,

    pub pgl_config: Account<'info, pgl1::state::PglConfig>,

    /// CHECK: validated against pgl_config.treasury.
    #[account(mut)]
    pub pgl_treasury: UncheckedAccount<'info>,

    pub pgl1_program: Program<'info, pgl1::program::Pgl1>,

    /// CHECK: game-store program, forwarded to game-store CPI.
    pub store_program: UncheckedAccount<'info>,

    /// CHECK: game-store authorized source program PDA.
    /// This is an existence check only — the game-store program validates
    /// the account data when it processes the CPI. Registry does not
    /// deserialize or trust data from this account.
    #[account(
        seeds = [b"authorized_source_program", pgl1_program.key().as_ref()],
        bump,
    )]
    pub store_authorized_source_program: UncheckedAccount<'info>,

    /// CHECK: game-store authorized registry program PDA.
    /// This is an existence check only — the game-store program validates
    /// the account data when it processes the CPI. Registry does not
    /// deserialize or trust data from this account.
    #[account(
        seeds = [b"authorized_registry_program", store_program.key().as_ref()],
        bump,
    )]
    pub store_authorized_registry_program: UncheckedAccount<'info>,

    /// CHECK: game-store config PDA, init by game-store CPI.
    #[account(mut)]
    pub store_game_store_config: UncheckedAccount<'info>,

    /// CHECK: self program ID, forwarded to game-store CPI.
    pub self_program: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateGameAndRegister<'info>>,
    game_id: String,
    metadata_uri: String,
    base_price: Option<u64>,
    mint_token: Option<Pubkey>,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), RegistryError::InvalidGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, RegistryError::InvalidGameId);
    require!(!metadata_uri.trim().is_empty(), RegistryError::InvalidMetadataUri);
    require!(metadata_uri.len() <= MAX_METADATA_URI_LEN, RegistryError::InvalidMetadataUri);
    require_keys_eq!(
        ctx.accounts.self_program.key(),
        crate::ID,
        RegistryError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.store_program.key(),
        GAME_STORE_PROGRAM_ID,
        RegistryError::InvalidStoreProgram
    );

    require_keys_eq!(
        ctx.accounts.pgl1_program.key(),
        ctx.accounts.config.pgl1_program,
        RegistryError::InvalidPgl1Program
    );

    let (expected_pgl_config, _) =
        Pubkey::find_program_address(&[pgl1::state::PGL_CONFIG_SEED], &ctx.accounts.pgl1_program.key());
    require_keys_eq!(
        ctx.accounts.pgl_config.key(),
        expected_pgl_config,
        RegistryError::InvalidPgl1Config
    );
    require_keys_eq!(
        ctx.accounts.pgl_treasury.key(),
        ctx.accounts.pgl_config.treasury,
        RegistryError::InvalidTreasury
    );

    let now = Clock::get()?.unix_timestamp;

    let publish_grant_account = ctx.remaining_accounts.first();
    let has_grant_remaining = publish_grant_account.is_some();

    let has_active_grant = load_optional_publish_grant(
        ctx.program_id,
        &ctx.accounts.publisher.key(),
        publish_grant_account,
    )?
    .map(|grant| grant.is_active_at(now))
    .unwrap_or(false);

    if !has_active_grant {
        require!(
            ctx.accounts.accepted_payment_token.active,
            RegistryError::PaymentTokenDisabled
        );
        require!(
            ctx.accounts.accepted_payment_token.fee_amount > 0,
            RegistryError::InvalidFeeAmount
        );

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.publisher_payment_account.to_account_info(),
            mint: ctx.accounts.payment_mint.to_account_info(),
            to: ctx.accounts.treasury_payment_account.to_account_info(),
            authority: ctx.accounts.publisher.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        transfer_checked(
            cpi_ctx,
            ctx.accounts.accepted_payment_token.fee_amount,
            ctx.accounts.payment_mint.decimals,
        )?;
    }

    let cpi_accounts = pgl1::cpi::accounts::CreateGame {
        creator: ctx.accounts.publisher.to_account_info(),
        pgl_config: ctx.accounts.pgl_config.to_account_info(),
        treasury: ctx.accounts.pgl_treasury.to_account_info(),
        creator_state: ctx.accounts.pgl_creator_state.to_account_info(),
        game: ctx.accounts.game.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.pgl1_program.to_account_info(), cpi_accounts);
    pgl1::cpi::create_game(cpi_ctx, game_id.clone(), metadata_uri)?;

    let game_pubkey = ctx.accounts.game.key();

    ctx.accounts.registry_game.game = game_pubkey;
    ctx.accounts.registry_game.game_id = game_id.clone();
    ctx.accounts.registry_game.registered_at = now;
    ctx.accounts.registry_game.status = GameStatus::Active;
    ctx.accounts.registry_game.bump = ctx.bumps.registry_game;

    invoke_init_game_store_config(
        &ctx.accounts.publisher.to_account_info(),
        &ctx.accounts.store_authorized_source_program,
        &ctx.accounts.pgl1_program.to_account_info(),
        &ctx.accounts.store_authorized_registry_program,
        &ctx.accounts.self_program,
        &ctx.accounts.game,
        &ctx.accounts.registry_game.to_account_info(),
        &ctx.accounts.store_game_store_config,
        &ctx.accounts.system_program.to_account_info(),
    )?;

    if let (Some(price), Some(mint)) = (base_price, mint_token) {
        require!(price > 0, RegistryError::InvalidPrice);

        let store_offset = if has_grant_remaining { 1 } else { 0 };
        let store_accepted_payment_token = ctx.remaining_accounts.get(store_offset).ok_or(error!(RegistryError::MissingStoreAccounts))?;
        let store_game_payment_option = ctx.remaining_accounts.get(store_offset + 1).ok_or(error!(RegistryError::MissingStoreAccounts))?;

        invoke_set_game_payment_option(
            &ctx.accounts.publisher.to_account_info(),
            &ctx.accounts.store_authorized_source_program,
            &ctx.accounts.pgl1_program.to_account_info(),
            &ctx.accounts.store_authorized_registry_program,
            &ctx.accounts.self_program,
            &ctx.accounts.game,
            &ctx.accounts.registry_game.to_account_info(),
            &ctx.accounts.store_game_store_config,
            &mint,
            store_accepted_payment_token,
            store_game_payment_option,
            &ctx.accounts.system_program.to_account_info(),
            price,
        )?;
    }

    emit!(GameRegistered {
        game: game_pubkey,
        game_id,
        status: GameStatus::Active.as_u8(),
    });

    Ok(())
}

fn invoke_init_game_store_config<'info>(
    publisher: &AccountInfo<'info>,
    authorized_source_program: &AccountInfo<'info>,
    source_program: &AccountInfo<'info>,
    authorized_registry_program: &AccountInfo<'info>,
    registry_program: &AccountInfo<'info>,
    game: &AccountInfo<'info>,
    registry_game: &AccountInfo<'info>,
    game_store_config: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    let mut data = Vec::with_capacity(9);
    data.extend_from_slice(&INIT_GAME_STORE_CONFIG_DISC);
    data.push(1u8);

    let accounts = vec![
        AccountMeta::new(publisher.key(), true),
        AccountMeta::new_readonly(authorized_source_program.key(), false),
        AccountMeta::new_readonly(source_program.key(), false),
        AccountMeta::new_readonly(authorized_registry_program.key(), false),
        AccountMeta::new_readonly(registry_program.key(), false),
        AccountMeta::new_readonly(game.key(), false),
        AccountMeta::new_readonly(registry_game.key(), false),
        AccountMeta::new(game_store_config.key(), false),
        AccountMeta::new_readonly(system_program.key(), false),
    ];

    let instruction = Instruction {
        program_id: GAME_STORE_PROGRAM_ID,
        accounts,
        data,
    };

    program::invoke(
        &instruction,
        &[
            publisher.clone(),
            authorized_source_program.clone(),
            source_program.clone(),
            authorized_registry_program.clone(),
            registry_program.clone(),
            game.clone(),
            registry_game.clone(),
            game_store_config.clone(),
            system_program.clone(),
        ],
    )?;

    Ok(())
}

fn invoke_set_game_payment_option<'info>(
    publisher: &AccountInfo<'info>,
    authorized_source_program: &AccountInfo<'info>,
    source_program: &AccountInfo<'info>,
    authorized_registry_program: &AccountInfo<'info>,
    registry_program: &AccountInfo<'info>,
    game: &AccountInfo<'info>,
    registry_game: &AccountInfo<'info>,
    game_store_config: &AccountInfo<'info>,
    mint: &Pubkey,
    accepted_payment_token: &AccountInfo<'info>,
    game_payment_option: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    base_price: u64,
) -> Result<()> {
    let mut data = Vec::with_capacity(17);
    data.extend_from_slice(&SET_GAME_PAYMENT_OPTION_DISC);
    data.extend_from_slice(&base_price.to_le_bytes());
    data.push(1u8);

    let accounts = vec![
        AccountMeta::new(publisher.key(), true),
        AccountMeta::new_readonly(authorized_source_program.key(), false),
        AccountMeta::new_readonly(source_program.key(), false),
        AccountMeta::new_readonly(authorized_registry_program.key(), false),
        AccountMeta::new_readonly(registry_program.key(), false),
        AccountMeta::new_readonly(game.key(), false),
        AccountMeta::new_readonly(registry_game.key(), false),
        AccountMeta::new_readonly(game_store_config.key(), false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(accepted_payment_token.key(), false),
        AccountMeta::new(game_payment_option.key(), false),
        AccountMeta::new_readonly(system_program.key(), false),
    ];

    let instruction = Instruction {
        program_id: GAME_STORE_PROGRAM_ID,
        accounts,
        data,
    };

    program::invoke(
        &instruction,
        &[
            publisher.clone(),
            authorized_source_program.clone(),
            source_program.clone(),
            authorized_registry_program.clone(),
            registry_program.clone(),
            game.clone(),
            registry_game.clone(),
            game_store_config.clone(),
            accepted_payment_token.clone(),
            game_payment_option.clone(),
            system_program.clone(),
        ],
    )?;

    Ok(())
}

fn load_optional_publish_grant(
    program_id: &Pubkey,
    publisher: &Pubkey,
    account_info: Option<&AccountInfo<'_>>,
) -> Result<Option<PublishGrant>> {
    let Some(account_info) = account_info else {
        return Ok(None);
    };

    let (expected_pda, _) =
        Pubkey::find_program_address(&[b"publish_grant", publisher.as_ref()], program_id);

    require_keys_eq!(
        account_info.key(),
        expected_pda,
        RegistryError::InvalidPublishGrantAccount
    );

    if account_info.owner == &system_program::ID || account_info.data_is_empty() {
        return Ok(None);
    }

    require_keys_eq!(
        *account_info.owner,
        *program_id,
        RegistryError::InvalidPublishGrantAccount
    );

    let data = account_info.try_borrow_data()?;
    let publish_grant_disc: [u8; 8] = [0x74, 0xbe, 0x9e, 0xc9, 0x0c, 0xd1, 0xb6, 0xf3];
    require!(
        data.len() >= 8 && data[..8] == publish_grant_disc,
        RegistryError::InvalidPublishGrantAccount
    );

    let mut data_slice: &[u8] = &data;
    let grant = PublishGrant::try_deserialize(&mut data_slice)?;

    Ok(Some(grant))
}
