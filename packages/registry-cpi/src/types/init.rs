use {
    crate::{
        state::Discriminator,
        types::common::{AssetItem, Range},
    },
    base::{
        helpers::{get_flag, set_flag},
        types::Result,
    },
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_serialize,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

// TODO: implement rest accounts
#[derive(AccountTryFrom, AccountMetas)]
#[repr(C)]
pub struct Accounts<'a> {
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub bump: &'a AccountInfo,

    #[account(writable)]
    pub config: &'a AccountInfo,

    #[account(writable)]
    pub user_counter: &'a AccountInfo,

    #[account(writable)]
    pub admin_rotation_state: &'a AccountInfo,

    pub revenue_mint: &'a AccountInfo,

    #[account(writable)]
    pub revenue_app_ata: &'a AccountInfo,
}

#[test_serialize(Discriminator::Init)]
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct InstructionData {
    pub flags: u8,
    pub rotation_timeout: [u8; 4],
    pub account_registration_fee: AssetItem,
    pub account_data_size_range: Range,
}

impl InstructionData {
    const ROTATION_TIMEOUT: u8 = 0;
    const ACCOUNT_REGISTRATION_FEE: u8 = 1;
    const ACCOUNT_DATA_SIZE_RANGE: u8 = 2;

    #[inline]
    pub fn is_rotation_timeout(&self) -> bool {
        get_flag(self.flags, Self::ROTATION_TIMEOUT)
    }

    #[inline]
    pub fn set_is_rotation_timeout(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::ROTATION_TIMEOUT, flag);
    }

    #[inline]
    pub fn is_account_registration_fee(&self) -> bool {
        get_flag(self.flags, Self::ACCOUNT_REGISTRATION_FEE)
    }

    #[inline]
    pub fn set_is_account_registration_fee(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::ACCOUNT_REGISTRATION_FEE, flag);
    }

    #[inline]
    pub fn is_account_data_size_range(&self) -> bool {
        get_flag(self.flags, Self::ACCOUNT_DATA_SIZE_RANGE)
    }

    #[inline]
    pub fn set_is_account_data_size_range(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::ACCOUNT_DATA_SIZE_RANGE, flag);
    }
}
