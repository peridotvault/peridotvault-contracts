use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::GameStoreError;

// --- Initialize ---

pub fn initialize_handler(
    ctx: Context<Initialize>,
    platform_fee_bps: u16,
    treasury: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.governance = ctx.accounts.authority.key();
    config.treasury = treasury;
    config.platform_fee_bps = platform_fee_bps;
    config.bump = ctx.bumps.config;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = StoreConfig::SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, StoreConfig>,
    pub system_program: Program<'info, System>,
}

// --- Set Price ---

pub fn set_price_handler(ctx: Context<SetPrice>, price: u64, currency: Pubkey) -> Result<()> {
    let price_account = &mut ctx.accounts.price_account;
    price_account.game = ctx.accounts.game.key();
    // In current state.rs, PriceAccount doesn't have a publisher field. 
    // It's linked to the PGC-1 game account which HAS a publisher.
    price_account.price = price;
    price_account.currency = currency;
    price_account.bump = ctx.bumps.price_account;
    Ok(())
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    /// CHECK: PGC-1 Game PDA
    pub game: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = publisher,
        space = PriceAccount::SPACE,
        seeds = [b"price", game.key().as_ref()],
        bump
    )]
    pub price_account: Account<'info, PriceAccount>,
    pub system_program: Program<'info, System>,
}

// --- Buy Game ---

pub fn buy_game_handler(ctx: Context<BuyGame>) -> Result<()> {
    let price = ctx.accounts.price_account.price;
    let config = &ctx.accounts.config;

    // 1. Calculate fees
    let platform_fee = (price as u128 * config.platform_fee_bps as u128 / 10000) as u64;
    let publisher_amount = price - platform_fee;

    // 2. Transfer Funds
    let currency = ctx.accounts.price_account.currency;
    if currency == anchor_lang::solana_program::system_program::ID {
        // Transfer Platform Fee
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.treasury.key(),
                platform_fee,
            ),
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Transfer Publisher Amount to Balance PDA
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.publisher_balance.key(),
                publisher_amount,
            ),
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.publisher_balance.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    } else {
        // Transfer Platform Fee (SPL)
        let cpi_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.treasury_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::transfer(cpi_ctx, platform_fee)?;

        // Transfer Publisher Amount to Vault (Publisher Balance Token Account)
        let cpi_accounts_pub = anchor_spl::token::Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.publisher_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_ctx_pub = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_pub);
        anchor_spl::token::transfer(cpi_ctx_pub, publisher_amount)?;
    }

    // 3. Update Publisher Balance
    let balance = &mut ctx.accounts.publisher_balance;
    balance.publisher = ctx.accounts.publisher.key();
    balance.token = currency;
    balance.amount += publisher_amount;

    // 4. CPI to PGC-1 to mint license (Fixed Sighash for mint_license: 0x39cc5d54a0f1fe34)
    let pgc_program = ctx.accounts.pgc_program.key();
    let mut data = vec![57, 204, 93, 84, 160, 241, 254, 52]; 
    data.extend_from_slice(&(0i64).to_le_bytes()); 

    let config_seeds = &[
        b"config".as_ref(),
        &[ctx.accounts.config.bump],
    ];
    let signer = &[&config_seeds[..]];

    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: pgc_program,
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.config.key(), true), // minter (Signer, readonly)
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.minter_pda.key(), false),
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.game.key(), false),
            anchor_lang::solana_program::instruction::AccountMeta::new(ctx.accounts.buyer.key(), true), // user (Signer, mut)
            anchor_lang::solana_program::instruction::AccountMeta::new(ctx.accounts.license_pda.key(), false),
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ],
        data,
    };

    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.config.to_account_info(),
            ctx.accounts.minter_pda.to_account_info(),
            ctx.accounts.game.to_account_info(),
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.license_pda.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct BuyGame<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, StoreConfig>,
    /// CHECK: Treasury from config
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ GameStoreError::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: PGC-1 Game PDA
    pub game: UncheckedAccount<'info>,
    #[account(
        seeds = [b"price", game.key().as_ref()],
        bump = price_account.bump
    )]
    pub price_account: Account<'info, PriceAccount>,
    
    /// CHECK: Publisher address
    pub publisher: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = PublisherBalanceAccount::SPACE,
        seeds = [b"balance", publisher.key().as_ref()],
        bump
    )]
    pub publisher_balance: Account<'info, PublisherBalanceAccount>,

    /// CHECK: PGC-1 Program
    pub pgc_program: UncheckedAccount<'info>,
    /// CHECK: PGC-1 Minter PDA
    pub minter_pda: UncheckedAccount<'info>,
    /// CHECK: PGC-1 License PDA
    #[account(mut)]
    pub license_pda: UncheckedAccount<'info>,

    /// CHECK: SPL Token Program (for SPL games)
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: Buyer Token Account (for SPL games)
    #[account(mut)]
    pub buyer_token_account: UncheckedAccount<'info>,
    /// CHECK: Treasury Token Account (for SPL games)
    #[account(mut)]
    pub treasury_token_account: UncheckedAccount<'info>,
    /// CHECK: Publisher Vault Token Account (for SPL games, owned by balance PDA)
    #[account(mut)]
    pub publisher_token_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

// --- Withdraw ---

pub fn withdraw_handler(ctx: Context<Withdraw>) -> Result<()> {
    let balance = &mut ctx.accounts.balance_account;
    let amount = balance.amount;
    let token = balance.token;
    balance.amount = 0;

    if token == anchor_lang::solana_program::system_program::ID {
        // SOL Withdraw
        **ctx.accounts.balance_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.publisher.to_account_info().try_borrow_mut_lamports()? += amount;
    } else {
        // SPL Token Withdraw
        let publisher_key = ctx.accounts.publisher.key();
        let seeds = &[
            b"balance",
            publisher_key.as_ref(),
            &[ctx.accounts.balance_account.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.publisher_token_account.to_account_info(),
            authority: ctx.accounts.balance_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(
        mut,
        seeds = [b"balance", publisher.key().as_ref()],
        bump = balance_account.bump,
        has_one = publisher @ GameStoreError::Unauthorized
    )]
    pub balance_account: Account<'info, PublisherBalanceAccount>,

    /// CHECK: SPL Token Program
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: Vault Token Account (owned by balance_account PDA)
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,
    /// CHECK: Publisher's destination Token Account
    #[account(mut)]
    pub publisher_token_account: UncheckedAccount<'info>,
}

// --- Set Platform Fee ---

pub fn set_platform_fee_handler(ctx: Context<SetPlatformFee>, bps: u16) -> Result<()> {
    ctx.accounts.config.platform_fee_bps = bps;
    Ok(())
}

#[derive(Accounts)]
pub struct SetPlatformFee<'info> {
    pub governance: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = governance @ GameStoreError::Unauthorized
    )]
    pub config: Account<'info, StoreConfig>,
}

// --- Set Treasury ---

pub fn set_treasury_handler(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
    ctx.accounts.config.treasury = treasury;
    Ok(())
}

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub governance: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = governance @ GameStoreError::Unauthorized
    )]
    pub config: Account<'info, StoreConfig>,
}

// --- Set Affiliate ---

pub fn set_affiliate_handler(_ctx: Context<SetAffiliate>, _affiliate: Pubkey, _bps: u16) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct SetAffiliate<'info> {
    pub governance: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = governance @ GameStoreError::Unauthorized
    )]
    pub config: Account<'info, StoreConfig>,
}

// --- Set Subscription ---

pub fn set_subscription_handler(_ctx: Context<SetSubscription>, _duration: i64, _price: u64) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct SetSubscription<'info> {
    pub governance: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = governance @ GameStoreError::Unauthorized
    )]
    pub config: Account<'info, StoreConfig>,
}
