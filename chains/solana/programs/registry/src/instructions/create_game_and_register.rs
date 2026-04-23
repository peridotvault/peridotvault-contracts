use anchor_lang::{prelude::*, solana_program::system_program};
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{
    errors::RegistryError,
    events::GameRegistered,
    state::{
        AcceptedPaymentToken, GameStatus, PublishGrant, RegistryConfig, RegistryGame, MAX_GAME_ID_LEN,
    },
};

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

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(
    ctx: Context<CreateGameAndRegister>,
    game_id: String,
    metadata_uri: String,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), RegistryError::InvalidGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, RegistryError::InvalidGameId);
    require!(!metadata_uri.trim().is_empty(), RegistryError::InvalidMetadataUri);

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

    let registry_game = &mut ctx.accounts.registry_game;
    registry_game.game = ctx.accounts.game.key();
    registry_game.game_id = game_id.clone();
    registry_game.registered_at = now;
    registry_game.status = GameStatus::Active;
    registry_game.bump = ctx.bumps.registry_game;

    emit!(GameRegistered {
        game: registry_game.game,
        game_id,
        status: GameStatus::Active.as_u8(),
    });

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
    let mut data_slice: &[u8] = &data;
    let grant = PublishGrant::try_deserialize(&mut data_slice)?;

    Ok(Some(grant))
}
