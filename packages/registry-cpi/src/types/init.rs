use {
    crate::{
        state::Discriminator,
        types::common::{AssetItem, Range},
    },
    base::types::{BitField, Result, Uint32},
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_serialize,
    macro_try_from::AccountTryFrom,
    macro_zc_serde::p_serde,
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
#[p_serde]
pub struct InstructionData {
    pub flags: BitField,
    pub rotation_timeout: Uint32,
    pub account_registration_fee: AssetItem,
    pub account_data_size_range: Range,
}

impl InstructionData {
    const ROTATION_TIMEOUT: u8 = 0;
    const ACCOUNT_REGISTRATION_FEE: u8 = 1;
    const ACCOUNT_DATA_SIZE_RANGE: u8 = 2;

    #[inline]
    pub fn get_rotation_timeout_flag(&self) -> bool {
        self.flags.get_flag(Self::ROTATION_TIMEOUT)
    }

    #[inline]
    pub fn set_rotation_timeout_flag(&mut self, x: bool) {
        self.flags.set_flag(Self::ROTATION_TIMEOUT, x);
    }

    #[inline]
    pub fn get_account_registration_fee_flag(&self) -> bool {
        self.flags.get_flag(Self::ACCOUNT_REGISTRATION_FEE)
    }

    #[inline]
    pub fn set_account_registration_fee_flag(&mut self, x: bool) {
        self.flags.set_flag(Self::ACCOUNT_REGISTRATION_FEE, x);
    }

    #[inline]
    pub fn get_account_data_size_range_flag(&self) -> bool {
        self.flags.get_flag(Self::ACCOUNT_DATA_SIZE_RANGE)
    }

    #[inline]
    pub fn set_account_data_size_range_flag(&mut self, x: bool) {
        self.flags.set_flag(Self::ACCOUNT_DATA_SIZE_RANGE, x);
    }
}
