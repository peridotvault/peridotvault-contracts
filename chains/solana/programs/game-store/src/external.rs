use quasar_lang::{
    cpi::{CpiCall, InstructionAccount},
    prelude::*,
};

pub const PGL1_ID: Address = address!("5YctJfQJ6qfYDchYKyHFyjeKa3dx8Z6kg5pt68yaZ6c3");
pub const REGISTRY_ID: Address = address!("8pgmtQDVpMX4FHmoCmWJCoB94RY56GKWUzo8f8e1Xfpo");

pub const AUTHORIZED_ACTOR_SEED: &[u8] = b"authorized_actor";
pub const LICENSE_SEED: &[u8] = b"license";

const PGL_GAME_DISC: [u8; 8] = [27, 90, 166, 125, 74, 100, 121, 18];
const PGL_AUTHORIZED_ACTOR_DISC: [u8; 8] = [155, 89, 1, 231, 51, 170, 32, 23];
const REGISTRY_GAME_DISC: [u8; 8] = [44, 59, 51, 135, 203, 140, 48, 151];
const MINT_LICENSE_DISC: [u8; 8] = [57, 204, 93, 84, 160, 241, 254, 52];

pub struct Pgl1Program;
impl Id for Pgl1Program {
    const ID: Address = PGL1_ID;
}

pub struct RegistryProgram;
impl Id for RegistryProgram {
    const ID: Address = REGISTRY_ID;
}

macro_rules! external_account_view {
    ($name:ident, $owner:ident, $disc:ident, $min_len:expr) => {
        quasar_lang::define_account!(pub struct $name => []);
        unsafe impl StaticView for $name {}
        impl CheckOwner for $name {
            #[inline(always)]
            fn check_owner(view: &AccountView) -> Result<(), ProgramError> {
                if !quasar_lang::keys_eq(view.owner(), &$owner) {
                    return Err(ProgramError::IllegalOwner);
                }
                Ok(())
            }
        }
        impl AccountCheck for $name {
            #[inline(always)]
            fn check(view: &AccountView) -> Result<(), ProgramError> {
                let data = unsafe { view.borrow_unchecked() };
                if data.len() < $min_len {
                    return Err(ProgramError::AccountDataTooSmall);
                }
                if data[..8] != $disc {
                    return Err(ProgramError::InvalidAccountData);
                }
                Ok(())
            }
        }
    };
}

external_account_view!(PglGame, PGL1_ID, PGL_GAME_DISC, 80);
external_account_view!(PglAuthorizedActor, PGL1_ID, PGL_AUTHORIZED_ACTOR_DISC, 42);
external_account_view!(RegistryGame, REGISTRY_ID, REGISTRY_GAME_DISC, 53);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RegistryGameStatus {
    Active,
    Suspended,
    Banned,
    Unknown,
}

#[inline(always)]
fn read_address(data: &[u8], offset: usize) -> Result<Address, ProgramError> {
    if data.len() < offset + 32 {
        return Err(ProgramError::AccountDataTooSmall);
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&data[offset..offset + 32]);
    Ok(Address::new_from_array(out))
}

impl PglGame {
    #[inline(always)]
    pub fn publisher(&self) -> Result<Address, ProgramError> {
        let data = unsafe { self.to_account_view().borrow_unchecked() };
        read_address(data, 8 + 32 + 8)
    }
}

impl PglAuthorizedActor {
    #[inline(always)]
    pub fn actor(&self) -> Result<Address, ProgramError> {
        let data = unsafe { self.to_account_view().borrow_unchecked() };
        read_address(data, 8)
    }

    #[inline(always)]
    pub fn active(&self) -> Result<bool, ProgramError> {
        let data = unsafe { self.to_account_view().borrow_unchecked() };
        if data.len() < 41 {
            return Err(ProgramError::AccountDataTooSmall);
        }
        Ok(data[40] != 0)
    }

    #[inline(always)]
    pub fn bump(&self) -> Result<u8, ProgramError> {
        let data = unsafe { self.to_account_view().borrow_unchecked() };
        if data.len() < 42 {
            return Err(ProgramError::AccountDataTooSmall);
        }
        Ok(data[41])
    }
}

impl RegistryGame {
    #[inline(always)]
    pub fn game(&self) -> Result<Address, ProgramError> {
        let data = unsafe { self.to_account_view().borrow_unchecked() };
        read_address(data, 8)
    }

    #[inline(always)]
    pub fn status(&self) -> Result<RegistryGameStatus, ProgramError> {
        let data = unsafe { self.to_account_view().borrow_unchecked() };
        // Quasar dynamic accounts store fixed fields before dynamic strings.
        let status_offset = 8 + 32 + 8;
        if data.len() <= status_offset {
            return Err(ProgramError::AccountDataTooSmall);
        }
        Ok(match data[status_offset] {
            0 => RegistryGameStatus::Active,
            1 => RegistryGameStatus::Suspended,
            2 => RegistryGameStatus::Banned,
            _ => RegistryGameStatus::Unknown,
        })
    }
}

#[inline(always)]
pub fn mint_license<'a>(
    pgl1_program: &'a impl AsAccountView,
    actor: &'a impl AsAccountView,
    holder: &'a impl AsAccountView,
    authorized_actor: &'a impl AsAccountView,
    game: &'a impl AsAccountView,
    license: &'a impl AsAccountView,
    system_program: &'a impl AsAccountView,
) -> Result<(), ProgramError> {
    let mut data = [0u8; 9];
    data[..8].copy_from_slice(&MINT_LICENSE_DISC);
    data[8] = 0;

    CpiCall::new(
        pgl1_program.to_account_view().address(),
        [
            InstructionAccount::readonly_signer(actor.to_account_view().address()),
            InstructionAccount::readonly(holder.to_account_view().address()),
            InstructionAccount::readonly(authorized_actor.to_account_view().address()),
            InstructionAccount::readonly(game.to_account_view().address()),
            InstructionAccount::writable(license.to_account_view().address()),
            InstructionAccount::readonly(system_program.to_account_view().address()),
        ],
        [
            actor.to_account_view(),
            holder.to_account_view(),
            authorized_actor.to_account_view(),
            game.to_account_view(),
            license.to_account_view(),
            system_program.to_account_view(),
        ],
        data,
    )
    .invoke()
}

#[inline(always)]
pub fn assert_active_registry_game(registry_game: &RegistryGame) -> Result<(), ProgramError> {
    if registry_game.status()? != RegistryGameStatus::Active {
        return Err(crate::errors::StoreError::GameNotActive.into());
    }
    Ok(())
}
