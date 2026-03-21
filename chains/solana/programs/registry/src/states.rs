use anchor_lang::prelude::*;

use crate::{
    constants::{
        MAX_ADMINS, MAX_FEE_EXEMPTIONS, MAX_GAME_ID_LEN, MAX_GAMES,
        MAX_REGISTRATION_FEE_OPTIONS,
    },
    errors::RegistryError,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct RegistryGame {
    pub game_id: String,
    pub contract_address: Pubkey,
    pub status: u8,
}

impl RegistryGame {
    pub const SPACE: usize = 4 + MAX_GAME_ID_LEN + 32 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct RegistrationFeeOption {
    pub payment_method: Pubkey,
    pub amount: u64,
}

impl RegistrationFeeOption {
    pub const SPACE: usize = 32 + 8;
}

#[account]
pub struct RegistryState {
    pub bump: u8,
    pub governance: Pubkey,
    pub treasury: Pubkey,
    pub factory: Pubkey,
    pub registration_fee_options: Vec<RegistrationFeeOption>,
    pub admins: Vec<Pubkey>,
    pub fee_exemptions: Vec<Pubkey>,
    pub games: Vec<RegistryGame>,
    pub all_game_ids: Vec<String>,
}

impl RegistryState {
    const FIXED_SPACE: usize = 8 + 1 + 32 + 32 + 32;
    const REGISTRATION_FEE_OPTIONS_SPACE: usize =
        4 + (MAX_REGISTRATION_FEE_OPTIONS * RegistrationFeeOption::SPACE);
    const ADMINS_SPACE: usize = 4 + (MAX_ADMINS * 32);
    const FEE_EXEMPTIONS_SPACE: usize = 4 + (MAX_FEE_EXEMPTIONS * 32);
    const GAMES_SPACE: usize = 4 + (MAX_GAMES * RegistryGame::SPACE);
    const GAME_IDS_SPACE: usize = 4 + (MAX_GAMES * (4 + MAX_GAME_ID_LEN));

    pub const SPACE: usize = Self::FIXED_SPACE
        + Self::REGISTRATION_FEE_OPTIONS_SPACE
        + Self::ADMINS_SPACE
        + Self::FEE_EXEMPTIONS_SPACE
        + Self::GAMES_SPACE
        + Self::GAME_IDS_SPACE;

    pub fn game_index(&self, game_id: &str) -> Option<usize> {
        self.games.iter().position(|game| game.game_id == game_id)
    }

    pub fn get_game(&self, game_id: &str) -> Option<&RegistryGame> {
        self.games.iter().find(|game| game.game_id == game_id)
    }

    pub fn is_admin(&self, account: &Pubkey) -> bool {
        self.admins.iter().any(|admin| admin == account)
    }

    pub fn is_fee_exempt(&self, account: &Pubkey) -> bool {
        self.fee_exemptions.iter().any(|entry| entry == account)
    }

    pub fn registration_fee_option(&self, payment_method: &Pubkey) -> Option<&RegistrationFeeOption> {
        self.registration_fee_options
            .iter()
            .find(|entry| &entry.payment_method == payment_method)
    }

    pub fn add_game(&mut self, game_id: String, contract_address: Pubkey, status: u8) -> Result<()> {
        require!(self.games.len() < MAX_GAMES, RegistryError::RegistryFull);

        self.games.push(RegistryGame {
            game_id: game_id.clone(),
            contract_address,
            status,
        });
        self.all_game_ids.push(game_id);
        Ok(())
    }

    pub fn set_admin(&mut self, account: Pubkey, is_admin: bool) -> Result<()> {
        match (self.is_admin(&account), is_admin) {
            (true, true) | (false, false) => Ok(()),
            (false, true) => {
                require!(self.admins.len() < MAX_ADMINS, RegistryError::AdminListFull);
                self.admins.push(account);
                Ok(())
            }
            (true, false) => {
                self.admins.retain(|admin| admin != &account);
                Ok(())
            }
        }
    }

    pub fn set_fee_exemption(&mut self, account: Pubkey, is_exempt: bool) -> Result<()> {
        match (self.is_fee_exempt(&account), is_exempt) {
            (true, true) | (false, false) => Ok(()),
            (false, true) => {
                require!(
                    self.fee_exemptions.len() < MAX_FEE_EXEMPTIONS,
                    RegistryError::FeeExemptionListFull
                );
                self.fee_exemptions.push(account);
                Ok(())
            }
            (true, false) => {
                self.fee_exemptions.retain(|entry| entry != &account);
                Ok(())
            }
        }
    }

    pub fn upsert_registration_fee_option(
        &mut self,
        payment_method: Pubkey,
        amount: u64,
    ) -> Result<bool> {
        if let Some(entry) = self
            .registration_fee_options
            .iter_mut()
            .find(|entry| entry.payment_method == payment_method)
        {
            if amount == 0 {
                self.registration_fee_options
                    .retain(|entry| entry.payment_method != payment_method);
                return Ok(false);
            }

            entry.amount = amount;
            return Ok(true);
        }

        if amount == 0 {
            return Ok(false);
        }

        require!(
            self.registration_fee_options.len() < MAX_REGISTRATION_FEE_OPTIONS,
            RegistryError::RegistrationFeeOptionLimitReached
        );
        self.registration_fee_options.push(RegistrationFeeOption {
            payment_method,
            amount,
        });
        Ok(true)
    }
}
