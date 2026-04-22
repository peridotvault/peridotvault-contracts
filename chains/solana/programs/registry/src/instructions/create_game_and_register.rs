use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program::invoke,
        system_program,
    },
};
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

    /// CHECK: expected to be created by CPI into the PGL-1 program
    #[account(mut)]
    pub game: UncheckedAccount<'info>,

    /// CHECK: forwarded to PGL-1 create_game CPI
    #[account(mut)]
    pub pgl_creator_state: UncheckedAccount<'info>,

    /// CHECK: forwarded to PGL-1 create_game CPI
    pub pgl_config: UncheckedAccount<'info>,

    /// CHECK: forwarded to PGL-1 create_game CPI
    #[account(mut)]
    pub pgl_treasury: UncheckedAccount<'info>,

    /// CHECK: external PGL-1 program
    pub pgl1_program: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct Pgl1CreateGameArgs {
    pub game_id: String,
    pub metadata_uri: String,
}

const PGL1_CREATE_GAME_DISCRIMINATOR: [u8; 8] = [124, 69, 75, 66, 184, 220, 72, 206];

pub(crate) fn handler(
    ctx: Context<CreateGameAndRegister>,
    game_id: String,
    metadata_uri: String,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), RegistryError::InvalidGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, RegistryError::InvalidGameId);
    require!(!metadata_uri.trim().is_empty(), RegistryError::InvalidMetadataUri);

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

    cpi_create_game_in_pgl1(
        &ctx.accounts.publisher.to_account_info(),
        &ctx.accounts.pgl_config.to_account_info(),
        &ctx.accounts.pgl_creator_state.to_account_info(),
        &ctx.accounts.game.to_account_info(),
        &ctx.accounts.pgl_treasury.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.pgl1_program.to_account_info(),
        game_id.clone(),
        metadata_uri,
    )?;

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

fn cpi_create_game_in_pgl1<'info>(
    creator: &AccountInfo<'info>,
    pgl_config: &AccountInfo<'info>,
    creator_state: &AccountInfo<'info>,
    game: &AccountInfo<'info>,
    treasury: &AccountInfo<'info>,
    system_program_ai: &AccountInfo<'info>,
    pgl1_program: &AccountInfo<'info>,
    game_id: String,
    metadata_uri: String,
) -> Result<()> {
    let mut data = Vec::with_capacity(128);
    data.extend_from_slice(&PGL1_CREATE_GAME_DISCRIMINATOR);

    let args = Pgl1CreateGameArgs {
        game_id,
        metadata_uri,
    };
    args.serialize(&mut data)?;

    let accounts = vec![
        AccountMeta::new(*creator.key, true),
        AccountMeta::new_readonly(*pgl_config.key, false),
        AccountMeta::new(*treasury.key, false),
        AccountMeta::new(*creator_state.key, false),
        AccountMeta::new(*game.key, false),
        AccountMeta::new_readonly(*system_program_ai.key, false),
    ];

    let ix = Instruction {
        program_id: *pgl1_program.key,
        accounts,
        data,
    };

    invoke(
        &ix,
        &[
            creator.clone(),
            pgl_config.clone(),
            treasury.clone(),
            creator_state.clone(),
            game.clone(),
            system_program_ai.clone(),
            pgl1_program.clone(),
        ],
    )?;

    Ok(())
}
