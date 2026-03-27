use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid status")]
    InvalidStatus,
    #[msg("Invalid authority")]
    InvalidAuthority,
}
