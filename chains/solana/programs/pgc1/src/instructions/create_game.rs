use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{Instruction, AccountMeta};
use anchor_lang::solana_program::program::invoke;
use crate::state::*;

pub fn create_game_handler(
    ctx: Context<CreateGame>,
    game_id: String,
    metadata_uri: String,
    initial_minter: Pubkey,
    price: u64,
    currency: Pubkey,
) -> Result<()> {
    // 1. Initialize Game Account
    let game_account = &mut ctx.accounts.game_account;
    game_account.game_id = game_id.clone();
    game_account.publisher = ctx.accounts.publisher.key();
    game_account.metadata_uri = metadata_uri;
    game_account.created_at = Clock::get()?.unix_timestamp;
    game_account.bump = ctx.bumps.game_account;

    // Flush data to account buffer so CPI programs (Store) can read the discriminator
    game_account.exit(ctx.program_id)?;

    // 2. Initialize Initial Minter
    let minter_account = &mut ctx.accounts.initial_minter_account;
    minter_account.game = game_account.key();
    minter_account.account = initial_minter;
    minter_account.is_authorized = true;
    minter_account.bump = ctx.bumps.initial_minter_account;

    // 3. Manual CPI to Registry.register_game
    // Discriminator: sha256("global:register_game").slice(0, 8)
    let mut reg_data = vec![122, 44, 95, 58, 89, 33, 40, 59];
    let game_id_bytes = game_id.as_bytes();
    reg_data.extend_from_slice(&(game_id_bytes.len() as u32).to_le_bytes());
    reg_data.extend_from_slice(game_id_bytes);
    reg_data.extend_from_slice(crate::ID.as_ref());
    reg_data.extend_from_slice(game_account.key().as_ref());

    let reg_ix = Instruction {
        program_id: ctx.accounts.registry_program.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.publisher.key(), true),
            AccountMeta::new(ctx.accounts.registry_game.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ],
        data: reg_data,
    };

    invoke(
        &reg_ix,
        &[
            ctx.accounts.publisher.to_account_info(),
            ctx.accounts.registry_game.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // 4. Manual CPI to Store.set_price
    // Discriminator: sha256("global:set_price").slice(0, 8)
    let mut store_data = vec![16, 19, 182, 8, 149, 83, 72, 181];
    store_data.extend_from_slice(&price.to_le_bytes());
    store_data.extend_from_slice(currency.as_ref());

    let store_ix = Instruction {
        program_id: ctx.accounts.store_program.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.publisher.key(), true),
            AccountMeta::new_readonly(game_account.key(), false),
            AccountMeta::new(ctx.accounts.price_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ],
        data: store_data,
    };

    invoke(
        &store_ix,
        &[
            ctx.accounts.publisher.to_account_info(),
            game_account.to_account_info(),
            ctx.accounts.price_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    Ok(())
}

