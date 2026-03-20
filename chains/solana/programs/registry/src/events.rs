use anchor_lang::prelude::*;

#[event]
pub struct RegistryInitialized {
    pub governance: Pubkey,
    pub treasury: Pubkey,
    pub factory: Pubkey,
    pub registration_fee: u64,
    pub registration_fee_token: Pubkey,
}

#[event]
pub struct GameRegistered {
    pub game_id: String,
    pub contract_address: Pubkey,
    pub publisher: Pubkey,
    pub status: u8,
    pub registered_by_factory: bool,
}

#[event]
pub struct GameStatusUpdated {
    pub game_id: String,
    pub old_status: u8,
    pub new_status: u8,
    pub admin: Pubkey,
}

#[event]
pub struct AdminUpdated {
    pub account: Pubkey,
    pub is_admin: bool,
}

#[event]
pub struct GovernanceUpdated {
    pub old_governance: Pubkey,
    pub new_governance: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub old_treasury: Pubkey,
    pub new_treasury: Pubkey,
}

#[event]
pub struct FactoryUpdated {
    pub old_factory: Pubkey,
    pub new_factory: Pubkey,
}

#[event]
pub struct RegistrationFeeUpdated {
    pub amount: u64,
    pub token: Pubkey,
}

#[event]
pub struct FeeExemptionUpdated {
    pub account: Pubkey,
    pub is_exempt: bool,
}
