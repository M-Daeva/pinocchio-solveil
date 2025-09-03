use {
    crate::{state::Discriminator, types::common::Range},
    base::{
        helpers::{get_flag, set_flag},
        types::Result,
    },
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_serialize,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

// TODO: fix warning: unused imports: `bool_as_bytes`, `pubkey_as_bytes`, `state::discriminator as DISCRIMINATOR`, and `u64_as_bytes`
// related to macro usage

#[derive(AccountTryFrom, AccountMetas)]
#[repr(C)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub config: &'a AccountInfo,

    #[account(writable)]
    pub admin_rotation_state: &'a AccountInfo,
}

#[test_serialize(Discriminator::UpdateConfig)]
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct InstructionData {
    flags: u8,
    pub admin: Pubkey,
    pub is_paused: u8,
    pub rotation_timeout: [u8; 4],
    pub registration_fee_amount: [u8; 8],
    pub data_size_range: Range,
}

// TODO: builder for setters?
impl InstructionData {
    const ADMIN: u8 = 0;
    const IS_PAUSED: u8 = 1;
    const ROTATION_TIMEOUT: u8 = 2;
    const REGISTRATION_FEE_AMOUNT: u8 = 3;
    const DATA_SIZE_RANGE: u8 = 4;

    #[inline]
    pub fn is_admin(&self) -> bool {
        get_flag(self.flags, Self::ADMIN)
    }

    #[inline]
    pub fn set_is_admin(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::ADMIN, flag);
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        get_flag(self.flags, Self::IS_PAUSED)
    }

    #[inline]
    pub fn set_is_paused(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::IS_PAUSED, flag);
    }

    #[inline]
    pub fn is_rotation_timeout(&self) -> bool {
        get_flag(self.flags, Self::ROTATION_TIMEOUT)
    }

    #[inline]
    pub fn set_is_rotation_timeout(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::ROTATION_TIMEOUT, flag);
    }

    #[inline]
    pub fn is_registration_fee_amount(&self) -> bool {
        get_flag(self.flags, Self::REGISTRATION_FEE_AMOUNT)
    }

    #[inline]
    pub fn set_is_registration_fee_amount(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::REGISTRATION_FEE_AMOUNT, flag);
    }

    #[inline]
    pub fn is_data_size_range(&self) -> bool {
        get_flag(self.flags, Self::DATA_SIZE_RANGE)
    }

    #[inline]
    pub fn set_is_data_size_range(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::DATA_SIZE_RANGE, flag);
    }
}
