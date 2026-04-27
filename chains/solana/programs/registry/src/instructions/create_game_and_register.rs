use crate::{
    errors::RegistryError,
    events::GameRegistered,
    external::{
        self, GameStoreProgram, Pgl1Program, PglConfigAccount, GAME_STORE_PROGRAM_ID,
        INIT_GAME_STORE_CONFIG_DISC, PGL_CONFIG_SEED, SET_GAME_PAYMENT_OPTION_DISC,
    },
    instructions::{read_option_address, read_option_u64, read_string},
    state::{
        AcceptedPaymentToken, GameStatus, RegistryConfig, RegistryGame,
        ACCEPTED_PAYMENT_TOKEN_SEED, MAX_GAME_ID_LEN, MAX_METADATA_URI_LEN, REGISTRY_CONFIG_SEED,
        REGISTRY_GAME_SEED,
    },
};
use quasar_lang::{
    cpi::{BufCpiCall, InstructionAccount},
    prelude::*,
    sysvars::Sysvar,
};
use quasar_spl::{InterfaceAccount, Mint, Token, TokenCpi, TokenInterface};

#[derive(Accounts)]
pub struct CreateGameAndRegister<'info> {
    pub publisher: &'info mut Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump)]
    pub config: &'info Account<RegistryConfig>,
    pub payment_mint: &'info InterfaceAccount<Mint>,
    #[account(seeds=[ACCEPTED_PAYMENT_TOKEN_SEED, payment_mint], bump=accepted_payment_token.bump)]
    pub accepted_payment_token: &'info Account<AcceptedPaymentToken>,
    #[account(mut, constraint=*publisher_payment_account.owner() == *publisher.address() @ RegistryError::Unauthorized, constraint=*publisher_payment_account.mint() == *payment_mint.address() @ RegistryError::PaymentTokenNotAllowed)]
    pub publisher_payment_account: &'info mut InterfaceAccount<Token>,
    #[account(mut, constraint=*treasury_payment_account.owner() == config.treasury @ RegistryError::InvalidTreasury, constraint=*treasury_payment_account.mint() == *payment_mint.address() @ RegistryError::PaymentTokenNotAllowed)]
    pub treasury_payment_account: &'info mut InterfaceAccount<Token>,
    #[account(init, mut, payer=publisher, space=RegistryGame::SPACE, seeds=[REGISTRY_GAME_SEED, game], bump)]
    pub registry_game: Account<RegistryGame<'info>>,
    #[account(mut)]
    pub game: &'info mut UncheckedAccount,
    #[account(mut)]
    pub pgl_creator_state: &'info mut UncheckedAccount,
    pub pgl_config: &'info Account<PglConfigAccount>,
    #[account(mut)]
    pub pgl_treasury: &'info UncheckedAccount,
    pub pgl1_program: &'info Program<Pgl1Program>,
    pub store_program: &'info Program<GameStoreProgram>,
    pub store_authorized_source_program: &'info UncheckedAccount,
    pub store_authorized_registry_program: &'info UncheckedAccount,
    #[account(mut)]
    pub store_game_store_config: &'info mut UncheckedAccount,
    pub token_program: &'info Interface<TokenInterface>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut CtxWithRemaining<'info, CreateGameAndRegister<'info>>,
) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let game_id = read_string(ctx.data, &mut offset, MAX_GAME_ID_LEN)?;
    let metadata_uri = read_string(ctx.data, &mut offset, MAX_METADATA_URI_LEN)?;
    let base_price = read_option_u64(ctx.data, &mut offset)?;
    let mint_token = read_option_address(ctx.data, &mut offset)?;
    require!(!game_id.trim().is_empty(), RegistryError::InvalidGameId);
    require!(
        !metadata_uri.trim().is_empty() && metadata_uri.len() <= MAX_METADATA_URI_LEN,
        RegistryError::InvalidMetadataUri
    );
    require_keys_eq!(
        *ctx.accounts.store_program.address(),
        GAME_STORE_PROGRAM_ID,
        RegistryError::InvalidStoreProgram
    );
    require_keys_eq!(
        *ctx.accounts.pgl1_program.address(),
        ctx.accounts.config.pgl1_program,
        RegistryError::InvalidPgl1Program
    );
    let (expected_pgl_config, _) = quasar_lang::pda::based_try_find_program_address(
        &[PGL_CONFIG_SEED],
        ctx.accounts.pgl1_program.address(),
    )?;
    require_keys_eq!(
        *ctx.accounts.pgl_config.address(),
        expected_pgl_config,
        RegistryError::InvalidPgl1Config
    );
    require_keys_eq!(
        *ctx.accounts.pgl_treasury.address(),
        ctx.accounts.pgl_config.treasury()?,
        RegistryError::InvalidTreasury
    );
    let now = Clock::get()?.unix_timestamp.get();

    if !ctx.accounts.accepted_payment_token.active.get() {
        return Err(RegistryError::PaymentTokenDisabled.into());
    }
    let fee_amount = ctx.accounts.accepted_payment_token.fee_amount.get();
    require!(fee_amount > 0, RegistryError::InvalidFeeAmount);
    require!(
        ctx.accounts.publisher_payment_account.amount() >= fee_amount,
        RegistryError::InsufficientFeeBalance
    );
    ctx.accounts
        .token_program
        .transfer_checked(
            ctx.accounts.publisher_payment_account,
            ctx.accounts.payment_mint,
            ctx.accounts.treasury_payment_account,
            ctx.accounts.publisher,
            fee_amount,
            ctx.accounts.payment_mint.decimals(),
        )
        .invoke()?;

    external::create_game(
        ctx.accounts.pgl1_program,
        ctx.accounts.publisher,
        ctx.accounts.pgl_config,
        ctx.accounts.pgl_treasury,
        ctx.accounts.pgl_creator_state,
        ctx.accounts.game,
        ctx.accounts.system_program,
        game_id,
        metadata_uri,
    )?;
    ctx.accounts.registry_game.set_inner(
        *ctx.accounts.game.address(),
        now,
        GameStatus::Active.into(),
        ctx.bumps.registry_game,
        game_id,
        ctx.accounts.publisher.to_account_view(),
        None,
    )?;

    invoke_init_game_store_config(ctx)?;
    if let (Some(price), Some(mint)) = (base_price, mint_token) {
        require!(price > 0, RegistryError::InvalidPrice);
        let rem = ctx.remaining_accounts();
        let accepted = rem.get(0).ok_or(RegistryError::MissingStoreAccounts)?;
        let option = rem.get(1).ok_or(RegistryError::MissingStoreAccounts)?;
        invoke_set_game_payment_option(ctx, &mint, &accepted, &option, price)?;
    }
    emit!(GameRegistered {
        game: *ctx.accounts.game.address(),
        game_id,
        publisher: *ctx.accounts.publisher.address(),
        status: GameStatus::Active.as_u8(),
        registered_at: now
    })?;
    Ok(())
}

fn invoke_init_game_store_config<'info>(
    ctx: &CtxWithRemaining<'info, CreateGameAndRegister<'info>>,
) -> Result<(), ProgramError> {
    let mut data = [0u8; 9];
    data[..8].copy_from_slice(&INIT_GAME_STORE_CONFIG_DISC);
    data[8] = 1;
    BufCpiCall::new(
        ctx.accounts.store_program.address(),
        [
            InstructionAccount::writable_signer(ctx.accounts.publisher.address()),
            InstructionAccount::readonly(ctx.accounts.store_authorized_source_program.address()),
            InstructionAccount::readonly(ctx.accounts.pgl1_program.address()),
            InstructionAccount::readonly(ctx.accounts.store_authorized_registry_program.address()),
            InstructionAccount::readonly(&crate::ID),
            InstructionAccount::readonly(ctx.accounts.game.address()),
            InstructionAccount::readonly(ctx.accounts.registry_game.address()),
            InstructionAccount::writable(ctx.accounts.store_game_store_config.address()),
            InstructionAccount::readonly(ctx.accounts.system_program.address()),
        ],
        [
            ctx.accounts.publisher.to_account_view(),
            ctx.accounts
                .store_authorized_source_program
                .to_account_view(),
            ctx.accounts.pgl1_program.to_account_view(),
            ctx.accounts
                .store_authorized_registry_program
                .to_account_view(),
            ctx.accounts.game.to_account_view(),
            ctx.accounts.registry_game.to_account_view(),
            ctx.accounts.store_game_store_config.to_account_view(),
            ctx.accounts.system_program.to_account_view(),
            ctx.accounts.system_program.to_account_view(),
        ],
        data,
        9,
    )
    .invoke()
}
fn invoke_set_game_payment_option<'info>(
    ctx: &CtxWithRemaining<'info, CreateGameAndRegister<'info>>,
    mint: &Address,
    accepted: &AccountView,
    option: &AccountView,
    price: u64,
) -> Result<(), ProgramError> {
    let mut data = [0u8; 17];
    data[..8].copy_from_slice(&SET_GAME_PAYMENT_OPTION_DISC);
    data[8..16].copy_from_slice(&price.to_le_bytes());
    data[16] = 1;
    BufCpiCall::new(
        ctx.accounts.store_program.address(),
        [
            InstructionAccount::writable_signer(ctx.accounts.publisher.address()),
            InstructionAccount::readonly(ctx.accounts.store_authorized_source_program.address()),
            InstructionAccount::readonly(ctx.accounts.pgl1_program.address()),
            InstructionAccount::readonly(ctx.accounts.store_authorized_registry_program.address()),
            InstructionAccount::readonly(&crate::ID),
            InstructionAccount::readonly(ctx.accounts.game.address()),
            InstructionAccount::readonly(ctx.accounts.registry_game.address()),
            InstructionAccount::readonly(ctx.accounts.store_game_store_config.address()),
            InstructionAccount::readonly(mint),
            InstructionAccount::readonly(accepted.address()),
            InstructionAccount::writable(option.address()),
            InstructionAccount::readonly(ctx.accounts.system_program.address()),
        ],
        [
            ctx.accounts.publisher.to_account_view(),
            ctx.accounts
                .store_authorized_source_program
                .to_account_view(),
            ctx.accounts.pgl1_program.to_account_view(),
            ctx.accounts
                .store_authorized_registry_program
                .to_account_view(),
            ctx.accounts.game.to_account_view(),
            ctx.accounts.registry_game.to_account_view(),
            ctx.accounts.store_game_store_config.to_account_view(),
            accepted,
            option,
            ctx.accounts.system_program.to_account_view(),
            ctx.accounts.system_program.to_account_view(),
            ctx.accounts.system_program.to_account_view(),
        ],
        data,
        17,
    )
    .invoke()
}
