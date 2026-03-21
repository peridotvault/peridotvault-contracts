use anchor_lang::{prelude::*, AccountDeserialize};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use pgc::{
    constants::LICENSE_SEED,
    cpi,
    cpi::accounts::MintLicense as MintLicenseCpiAccounts,
    program::Pgc1,
    states::{GameState as PgcGameState, LicenseAccount as PgcLicenseAccount, MinterAuthority},
};
use registry::{
    constants::STATUS_APPROVED,
    states::RegistryState,
};

use crate::{
    constants::STORE_STATE_SEED,
    errors::GameStoreError,
    events::GamePurchased,
    states::StoreState,
};

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct BuyGame<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump
    )]
    pub store_state: Account<'info, StoreState>,

    #[account(address = store_state.registry)]
    pub registry_state: Account<'info, RegistryState>,

    pub pgc_program: Program<'info, Pgc1>,

    #[account(mut)]
    pub pgc_game_state: Account<'info, PgcGameState>,

    /// CHECK: validated by PGC CPI
    pub game_authority: UncheckedAccount<'info>,

    #[account(
        constraint = store_minter_auth.game == pgc_game_state.key() @ GameStoreError::Unauthorized,
        constraint = store_minter_auth.account == store_state.key() @ GameStoreError::Unauthorized,
        constraint = store_minter_auth.is_authorized @ GameStoreError::Unauthorized
    )]
    pub store_minter_auth: Account<'info, MinterAuthority>,

    /// CHECK: validated manually against the expected PGC PDA and by PGC CPI
    #[account(mut)]
    pub license_account: UncheckedAccount<'info>,

    /// CHECK: validated by PGC CPI ATA constraints
    #[account(mut)]
    pub user_game_token_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub game_mint: InterfaceAccount<'info, anchor_spl::token_interface::Mint>,

    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = buyer_payment_token_account.owner == buyer.key() @ GameStoreError::InvalidBuyerTokenAccount,
        constraint = buyer_payment_token_account.mint == payment_mint.key() @ GameStoreError::InvalidPaymentMint
    )]
    pub buyer_payment_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = treasury_token_account.owner == store_state.treasury @ GameStoreError::InvalidTreasuryTokenAccount,
        constraint = treasury_token_account.mint == payment_mint.key() @ GameStoreError::InvalidPaymentMint
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = payment_mint,
        associated_token::authority = store_state,
        associated_token::token_program = payment_token_program
    )]
    pub store_vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub payment_token_program: Interface<'info, TokenInterface>,
    pub license_token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyGame>, game_id: String) -> Result<()> {
    let registry_game = ctx
        .accounts
        .registry_state
        .get_game(&game_id)
        .ok_or(error!(GameStoreError::GameNotFound))?;

    require!(registry_game.status == STATUS_APPROVED, GameStoreError::GameNotApproved);
    require_keys_eq!(
        registry_game.contract_address,
        ctx.accounts.pgc_game_state.key(),
        GameStoreError::ContractAddressMismatch
    );
    require_keys_eq!(
        ctx.accounts.pgc_game_state.mint,
        ctx.accounts.game_mint.key(),
        GameStoreError::ContractAddressMismatch
    );

    let price_config = ctx
        .accounts
        .store_state
        .price_config(&game_id)
        .cloned()
        .ok_or(error!(GameStoreError::PriceConfigNotFound))?;

    require_keys_eq!(
        price_config.currency,
        ctx.accounts.payment_mint.key(),
        GameStoreError::InvalidPaymentMint
    );
    require_keys_eq!(
        ctx.accounts.store_vault_token_account.owner,
        ctx.accounts.store_state.key(),
        GameStoreError::InvalidStoreVaultTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.store_vault_token_account.mint,
        ctx.accounts.payment_mint.key(),
        GameStoreError::InvalidPaymentMint
    );

    let (expected_license_account, _) = Pubkey::find_program_address(
        &[
            LICENSE_SEED,
            ctx.accounts.pgc_game_state.key().as_ref(),
            ctx.accounts.buyer.key().as_ref(),
        ],
        &ctx.accounts.pgc_program.key(),
    );
    require_keys_eq!(
        ctx.accounts.license_account.key(),
        expected_license_account,
        GameStoreError::InvalidLicenseAccount
    );

    if ctx.accounts.license_account.owner == &ctx.accounts.pgc_program.key()
        && !ctx.accounts.license_account.data_is_empty()
    {
        let mut license_data: &[u8] = &ctx.accounts.license_account.data.borrow();
        let license = PgcLicenseAccount::try_deserialize(&mut license_data)
            .map_err(|_| error!(GameStoreError::InvalidLicenseAccount))?;
        let now = Clock::get()?.unix_timestamp;
        if license.game == ctx.accounts.pgc_game_state.key()
            && license.user == ctx.accounts.buyer.key()
            && license.is_valid(now)
        {
            return err!(GameStoreError::AlreadyOwnsValidLicense);
        }
    }

    let final_price = StoreState::final_price(&price_config);
    let platform_fee =
        ((u128::from(final_price) * u128::from(ctx.accounts.store_state.platform_fee_bps)) / 10_000)
            as u64;
    let publisher_revenue = final_price.saturating_sub(platform_fee);
    let publisher = ctx.accounts.pgc_game_state.publisher;

    if platform_fee > 0 {
        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.payment_token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.buyer_payment_token_account.to_account_info(),
                    mint: ctx.accounts.payment_mint.to_account_info(),
                    to: ctx.accounts.treasury_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            platform_fee,
            ctx.accounts.payment_mint.decimals,
        )?;
    }

    if publisher_revenue > 0 {
        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.payment_token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.buyer_payment_token_account.to_account_info(),
                    mint: ctx.accounts.payment_mint.to_account_info(),
                    to: ctx.accounts.store_vault_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            publisher_revenue,
            ctx.accounts.payment_mint.decimals,
        )?;
    }

    let store_state = &mut ctx.accounts.store_state;
    store_state.credit_publisher_balance(publisher, price_config.currency, publisher_revenue)?;

    let store_signer_seeds: &[&[u8]] = &[STORE_STATE_SEED, &[ctx.accounts.store_state.bump]];

    cpi::mint_license(
        CpiContext::new_with_signer(
            ctx.accounts.pgc_program.to_account_info(),
            MintLicenseCpiAccounts {
                payer: ctx.accounts.buyer.to_account_info(),
                signer: ctx.accounts.store_state.to_account_info(),
                game_state: ctx.accounts.pgc_game_state.to_account_info(),
                user: ctx.accounts.buyer.to_account_info(),
                game_authority: ctx.accounts.game_authority.to_account_info(),
                minter_auth: ctx.accounts.store_minter_auth.to_account_info(),
                mint: ctx.accounts.game_mint.to_account_info(),
                license_account: ctx.accounts.license_account.to_account_info(),
                user_token_account: ctx.accounts.user_game_token_account.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                token_program: ctx.accounts.license_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[store_signer_seeds],
        ),
        0,
    )?;

    emit!(GamePurchased {
        game_id,
        buyer: ctx.accounts.buyer.key(),
        publisher,
        currency: price_config.currency,
        final_price,
        platform_fee,
        publisher_revenue,
    });

    Ok(())
}
