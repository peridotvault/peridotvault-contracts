pub mod add_payment_token;
pub mod close_registry_game;
pub mod create_game_and_register;
pub mod initialize_registry;
pub mod remove_payment_token;
pub mod set_publish_grant;
pub mod set_treasury;
pub mod update_game_status;
pub mod update_payment_token;

pub use add_payment_token::*;
pub use close_registry_game::*;
pub use create_game_and_register::*;
pub use initialize_registry::*;
pub use remove_payment_token::*;
pub use set_publish_grant::*;
pub use set_treasury::*;
pub use update_game_status::*;
pub use update_payment_token::*;

use quasar_lang::prelude::*;

pub(crate) fn read_u8(data: &[u8], offset: &mut usize) -> Result<u8, ProgramError> {
    if data.len() < *offset + 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let v = data[*offset];
    *offset += 1;
    Ok(v)
}
pub(crate) fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32, ProgramError> {
    if data.len() < *offset + 4 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let v = u32::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    Ok(v)
}
pub(crate) fn read_u64(data: &[u8], offset: &mut usize) -> Result<u64, ProgramError> {
    if data.len() < *offset + 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let v = u64::from_le_bytes([
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
    Ok(v)
}
pub(crate) fn read_i64(data: &[u8], offset: &mut usize) -> Result<i64, ProgramError> {
    if data.len() < *offset + 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let v = i64::from_le_bytes([
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
    Ok(v)
}
pub(crate) fn read_address(data: &[u8], offset: &mut usize) -> Result<Address, ProgramError> {
    if data.len() < *offset + 32 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut b = [0u8; 32];
    b.copy_from_slice(&data[*offset..*offset + 32]);
    *offset += 32;
    Ok(Address::new_from_array(b))
}
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
pub(crate) fn read_option_u64(
    data: &[u8],
    offset: &mut usize,
) -> Result<Option<u64>, ProgramError> {
    match read_u8(data, offset)? {
        0 => Ok(None),
        1 => Ok(Some(read_u64(data, offset)?)),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
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
pub(crate) fn read_string<'a>(
    data: &'a [u8],
    offset: &mut usize,
    max_len: usize,
) -> Result<&'a str, ProgramError> {
    let len = read_u32(data, offset)? as usize;
    if len > max_len || data.len() < *offset + len {
        return Err(ProgramError::InvalidInstructionData);
    }
    let bytes = &data[*offset..*offset + len];
    *offset += len;
    core::str::from_utf8(bytes).map_err(|_| ProgramError::InvalidInstructionData)
}
