pub mod add_authorized_actor;
pub mod close_authorized_actor;
pub mod close_creator_state;
pub mod create_game;
pub mod deactivate_authorized_actor;
pub mod initialize_pgl;
pub mod mint_license;
pub mod renew_license;
pub mod set_authority;
pub mod set_create_game_fee;
pub mod set_metadata_uri;
pub mod set_publisher;
pub mod set_treasury;

pub use add_authorized_actor::*;
pub use close_authorized_actor::*;
pub use close_creator_state::*;
pub use create_game::*;
pub use deactivate_authorized_actor::*;
pub use initialize_pgl::*;
pub use mint_license::*;
pub use renew_license::*;
pub use set_authority::*;
pub use set_create_game_fee::*;
pub use set_metadata_uri::*;
pub use set_publisher::*;
pub use set_treasury::*;

use quasar_lang::prelude::*;

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
pub(crate) fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32, ProgramError> {
    if data.len() < *offset + 4 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let value = u32::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
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
