use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_GAME_ID_LEN, MAX_PRICE_CONFIGS, MAX_PUBLISHER_BALANCES},
    errors::GameStoreError,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct PriceConfig {
    pub game_id: String,
    pub price: u64,
    pub currency: Pubkey,
    pub discount_bps: u16,
}

impl PriceConfig {
    pub const SPACE: usize = 4 + MAX_GAME_ID_LEN + 8 + 32 + 2;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublisherBalance {
    pub publisher: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
}

impl PublisherBalance {
    pub const SPACE: usize = 32 + 32 + 8;
}

#[account]
pub struct StoreState {
    pub bump: u8,
    pub registry: Pubkey,
    pub governance: Pubkey,
    pub treasury: Pubkey,
    pub platform_fee_bps: u16,
    pub prices: Vec<PriceConfig>,
    pub publisher_balances: Vec<PublisherBalance>,
}

impl StoreState {
    const FIXED_SPACE: usize = 8 + 1 + 32 + 32 + 32 + 2;
    const PRICES_SPACE: usize = 4 + (MAX_PRICE_CONFIGS * PriceConfig::SPACE);
    const BALANCES_SPACE: usize = 4 + (MAX_PUBLISHER_BALANCES * PublisherBalance::SPACE);

    pub const SPACE: usize = Self::FIXED_SPACE + Self::PRICES_SPACE + Self::BALANCES_SPACE;

    pub fn price_index(&self, game_id: &str) -> Option<usize> {
        self.prices.iter().position(|entry| entry.game_id == game_id)
    }

    pub fn price_config(&self, game_id: &str) -> Option<&PriceConfig> {
        self.prices.iter().find(|entry| entry.game_id == game_id)
    }

    pub fn upsert_price(&mut self, game_id: String, price: u64, currency: Pubkey) -> Result<()> {
        match self.price_index(&game_id) {
            Some(index) => {
                self.prices[index].price = price;
                self.prices[index].currency = currency;
            }
            None => {
                require!(
                    self.prices.len() < MAX_PRICE_CONFIGS,
                    GameStoreError::PriceConfigLimitReached
                );
                self.prices.push(PriceConfig {
                    game_id,
                    price,
                    currency,
                    discount_bps: 0,
                });
            }
        }
        Ok(())
    }

    pub fn set_discount(&mut self, game_id: &str, discount_bps: u16) -> Result<()> {
        let index = self
            .price_index(game_id)
            .ok_or(error!(GameStoreError::PriceConfigNotFound))?;
        self.prices[index].discount_bps = discount_bps;
        Ok(())
    }

    pub fn final_price(price: &PriceConfig) -> u64 {
        let discount = (u128::from(price.price) * u128::from(price.discount_bps)) / 10_000;
        (u128::from(price.price) - discount) as u64
    }

    pub fn publisher_balance(&self, publisher: &Pubkey, token: &Pubkey) -> u64 {
        self.publisher_balances
            .iter()
            .find(|entry| &entry.publisher == publisher && &entry.token == token)
            .map(|entry| entry.amount)
            .unwrap_or(0)
    }

    pub fn credit_publisher_balance(
        &mut self,
        publisher: Pubkey,
        token: Pubkey,
        amount: u64,
    ) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }

        if let Some(entry) = self
            .publisher_balances
            .iter_mut()
            .find(|entry| entry.publisher == publisher && entry.token == token)
        {
            entry.amount = entry.amount.saturating_add(amount);
            return Ok(());
        }

        require!(
            self.publisher_balances.len() < MAX_PUBLISHER_BALANCES,
            GameStoreError::PublisherBalanceLimitReached
        );
        self.publisher_balances.push(PublisherBalance {
            publisher,
            token,
            amount,
        });
        Ok(())
    }

    pub fn take_publisher_balance(&mut self, publisher: Pubkey, token: Pubkey) -> Result<u64> {
        let entry = self
            .publisher_balances
            .iter_mut()
            .find(|entry| entry.publisher == publisher && entry.token == token)
            .ok_or(error!(GameStoreError::PublisherBalanceNotFound))?;

        require!(entry.amount > 0, GameStoreError::EmptyPublisherBalance);

        let amount = entry.amount;
        entry.amount = 0;
        Ok(amount)
    }
}
