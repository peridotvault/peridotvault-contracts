use quasar_lang::prelude::*;

pub struct RegistryInitialized {
    pub authority: Address,
    pub treasury: Address,
    pub pgl1_program: Address,
}
pub struct TreasuryUpdated {
    pub old_treasury: Address,
    pub new_treasury: Address,
}
pub struct PaymentTokenAdded {
    pub mint: Address,
    pub fee_amount: u64,
}
pub struct PaymentTokenUpdated {
    pub mint: Address,
    pub old_active: bool,
    pub new_active: bool,
    pub old_fee_amount: u64,
    pub new_fee_amount: u64,
}
pub struct PaymentTokenRemoved {
    pub mint: Address,
}
pub struct PublishGrantCreated {
    pub publisher: Address,
    pub expired_at: Option<i64>,
}
pub struct PublishGrantUpdated {
    pub publisher: Address,
    pub old_expired_at: Option<i64>,
    pub new_expired_at: Option<i64>,
}
pub struct GameRegistered<'a> {
    pub game: Address,
    pub game_id: &'a str,
    pub publisher: Address,
    pub status: u8,
    pub registered_at: i64,
}
pub struct GameStatusUpdated {
    pub game: Address,
    pub old_status: u8,
    pub new_status: u8,
    pub authority: Address,
}
pub struct GameClosed<'a> {
    pub game: Address,
    pub game_id: &'a str,
    pub closed_by: Address,
}

macro_rules! impl_noop_emit {
    ($($name:ident $(<$lt:lifetime>)?),* $(,)?) => {$(impl$(<$lt>)? $name$(<$lt>)? { #[inline(always)] pub fn emit_log(self) -> Result<(), ProgramError> { Ok(()) } })*};
}
impl_noop_emit!(
    RegistryInitialized,
    TreasuryUpdated,
    PaymentTokenAdded,
    PaymentTokenUpdated,
    PaymentTokenRemoved,
    PublishGrantCreated,
    PublishGrantUpdated,
    GameRegistered<'a>,
    GameStatusUpdated,
    GameClosed<'a>
);
