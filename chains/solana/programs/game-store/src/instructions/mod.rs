pub mod add_authorized_program;
pub mod add_payment_token;
pub mod buy_game;
pub mod clear_discount;
pub mod init_game_store_config;
pub mod initialize_store;
pub mod remove_game_payment_option;
pub mod set_default_referral;
pub mod set_discount;
pub mod set_game_payment_option;
pub mod set_game_store_active;
pub mod set_max_referral;
pub mod set_platform_fee;
pub mod set_referral_bps;
pub mod set_store_actor;
pub mod set_treasury;
pub mod update_authorized_program;
pub mod update_payment_token;

pub use add_authorized_program::*;
pub use add_payment_token::*;
pub use buy_game::*;
pub use clear_discount::*;
pub use init_game_store_config::*;
pub use initialize_store::*;
pub use remove_game_payment_option::*;
pub use set_default_referral::*;
pub use set_discount::*;
pub use set_game_payment_option::*;
pub use set_game_store_active::*;
pub use set_max_referral::*;
pub use set_platform_fee::*;
pub use set_referral_bps::*;
pub use set_store_actor::*;
pub use set_treasury::*;
pub use update_authorized_program::*;
pub use update_payment_token::*;

use quasar_lang::prelude::*;

#[inline(always)]
pub(crate) fn read_bool(data: &[u8], offset: &mut usize) -> Result<bool, ProgramError> {
    if data.len() < *offset + 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let value = data[*offset] != 0;
    *offset += 1;
    Ok(value)
}

#[inline(always)]
pub(crate) fn read_u8(data: &[u8], offset: &mut usize) -> Result<u8, ProgramError> {
    if data.len() < *offset + 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let value = data[*offset];
    *offset += 1;
    Ok(value)
}

#[inline(always)]
pub(crate) fn read_u16(data: &[u8], offset: &mut usize) -> Result<u16, ProgramError> {
    if data.len() < *offset + 2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let value = u16::from_le_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    Ok(value)
}

#[inline(always)]
pub(crate) fn read_i64(data: &[u8], offset: &mut usize) -> Result<i64, ProgramError> {
    if data.len() < *offset + 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let value = i64::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
        data[*offset + 4],
        data[*offset + 5],
        data[*offset + 6],
        data[*offset + 7],
    ]);
    *offset += 8;
    Ok(value)
}

#[inline(always)]
pub(crate) fn read_u64(data: &[u8], offset: &mut usize) -> Result<u64, ProgramError> {
    if data.len() < *offset + 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let value = u64::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
        data[*offset + 4],
        data[*offset + 5],
        data[*offset + 6],
        data[*offset + 7],
    ]);
    *offset += 8;
    Ok(value)
}

#[inline(always)]
pub(crate) fn read_address(data: &[u8], offset: &mut usize) -> Result<Address, ProgramError> {
    if data.len() < *offset + 32 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data[*offset..*offset + 32]);
    *offset += 32;
    Ok(Address::new_from_array(bytes))
}

#[inline(always)]
pub(crate) fn read_option_u8(data: &[u8], offset: &mut usize) -> Result<Option<u8>, ProgramError> {
    match read_u8(data, offset)? {
        0 => Ok(None),
        1 => Ok(Some(read_u8(data, offset)?)),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[inline(always)]
pub(crate) fn read_option_u16(
    data: &[u8],
    offset: &mut usize,
) -> Result<Option<u16>, ProgramError> {
    match read_u8(data, offset)? {
        0 => Ok(None),
        1 => Ok(Some(read_u16(data, offset)?)),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[inline(always)]
pub(crate) fn read_option_i64(
    data: &[u8],
    offset: &mut usize,
) -> Result<Option<i64>, ProgramError> {
    match read_u8(data, offset)? {
        0 => Ok(None),
        1 => Ok(Some(read_i64(data, offset)?)),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[inline(always)]
pub(crate) fn read_option_address(
    data: &[u8],
    offset: &mut usize,
) -> Result<Option<Address>, ProgramError> {
    match read_u8(data, offset)? {
        0 => Ok(None),
        1 => Ok(Some(read_address(data, offset)?)),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
