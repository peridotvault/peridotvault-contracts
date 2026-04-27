use quasar_lang::{
    cpi::{BufCpiCall, InstructionAccount},
    prelude::*,
};

pub const PGL1_ID: Address = address!("5YctJfQJ6qfYDchYKyHFyjeKa3dx8Z6kg5pt68yaZ6c3");
pub const GAME_STORE_PROGRAM_ID: Address = address!("6gTd8TQ9NiC7yxBfGWBzH1aWdk77fg779nUJhYTrEsPd");
pub const PGL_CONFIG_SEED: &[u8] = b"pgl_config";

const PGL_CONFIG_DISC: [u8; 8] = [152, 183, 211, 24, 96, 186, 93, 22];
const PGL_GAME_DISC: [u8; 8] = [27, 90, 166, 125, 74, 100, 121, 18];
const CREATE_GAME_DISC: [u8; 8] = [124, 69, 75, 66, 184, 220, 72, 206];
pub const INIT_GAME_STORE_CONFIG_DISC: [u8; 8] = [0x7e, 0xd2, 0xfe, 0x0b, 0x7c, 0x57, 0xe4, 0xa3];
pub const SET_GAME_PAYMENT_OPTION_DISC: [u8; 8] = [0x23, 0x98, 0x38, 0xe4, 0x80, 0xa1, 0xa2, 0xae];

pub struct Pgl1Program;
impl Id for Pgl1Program {
    const ID: Address = PGL1_ID;
}

pub struct GameStoreProgram;
impl Id for GameStoreProgram {
    const ID: Address = GAME_STORE_PROGRAM_ID;
}

macro_rules! account_view {
    ($name:ident, $disc:ident, $min:expr) => {
        quasar_lang::define_account!(pub struct $name => []);
        unsafe impl StaticView for $name {}
        impl CheckOwner for $name {
            fn check_owner(view: &AccountView) -> Result<(), ProgramError> {
                if !quasar_lang::keys_eq(view.owner(), &PGL1_ID) { return Err(ProgramError::IllegalOwner); }
                Ok(())
            }
        }
        impl AccountCheck for $name {
            fn check(view: &AccountView) -> Result<(), ProgramError> {
                let data = unsafe { view.borrow_unchecked() };
                if data.len() < $min { return Err(ProgramError::AccountDataTooSmall); }
                if data[..8] != $disc { return Err(ProgramError::InvalidAccountData); }
                Ok(())
            }
        }
    }
}

account_view!(PglConfigAccount, PGL_CONFIG_DISC, 81);
account_view!(PglGame, PGL_GAME_DISC, 89);

fn read_address(data: &[u8], offset: usize) -> Result<Address, ProgramError> {
    if data.len() < offset + 32 {
        return Err(ProgramError::AccountDataTooSmall);
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&data[offset..offset + 32]);
    Ok(Address::new_from_array(out))
}

impl PglConfigAccount {
    pub fn treasury(&self) -> Result<Address, ProgramError> {
        let d = unsafe { self.to_account_view().borrow_unchecked() };
        read_address(d, 8 + 32)
    }
}
impl PglGame {
    pub fn publisher(&self) -> Result<Address, ProgramError> {
        let d = unsafe { self.to_account_view().borrow_unchecked() };
        read_address(d, 8 + 32 + 8)
    }
}

pub fn create_game<'a>(
    pgl1_program: &'a impl AsAccountView,
    publisher: &'a impl AsAccountView,
    pgl_config: &'a impl AsAccountView,
    pgl_treasury: &'a impl AsAccountView,
    pgl_creator_state: &'a impl AsAccountView,
    game: &'a impl AsAccountView,
    system_program: &'a impl AsAccountView,
    game_id: &str,
    metadata_uri: &str,
) -> Result<(), ProgramError> {
    let mut data = [0u8; 8 + 4 + 64 + 4 + 256];
    let mut offset = 0usize;
    data[..8].copy_from_slice(&CREATE_GAME_DISC);
    offset += 8;
    data[offset..offset + 4].copy_from_slice(&(game_id.len() as u32).to_le_bytes());
    offset += 4;
    data[offset..offset + game_id.len()].copy_from_slice(game_id.as_bytes());
    offset += game_id.len();
    data[offset..offset + 4].copy_from_slice(&(metadata_uri.len() as u32).to_le_bytes());
    offset += 4;
    data[offset..offset + metadata_uri.len()].copy_from_slice(metadata_uri.as_bytes());
    offset += metadata_uri.len();
    BufCpiCall::new(
        pgl1_program.to_account_view().address(),
        [
            InstructionAccount::writable_signer(publisher.to_account_view().address()),
            InstructionAccount::readonly(pgl_config.to_account_view().address()),
            InstructionAccount::writable(pgl_treasury.to_account_view().address()),
            InstructionAccount::writable(pgl_creator_state.to_account_view().address()),
            InstructionAccount::writable(game.to_account_view().address()),
            InstructionAccount::readonly(system_program.to_account_view().address()),
        ],
        [
            publisher.to_account_view(),
            pgl_config.to_account_view(),
            pgl_treasury.to_account_view(),
            pgl_creator_state.to_account_view(),
            game.to_account_view(),
            system_program.to_account_view(),
        ],
        data,
        offset,
    )
    .invoke()
}
