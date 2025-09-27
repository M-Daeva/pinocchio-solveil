use {
    crate::{state::Discriminator, types::common::Range},
    base::{
        traits::DataLen,
        types::{BitField, Result, Uint32, Uint64},
    },
    bytemuck::{Pod, Zeroable},
    macro_optional_flag::OptionFlag,
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

// TODO: fix warning: unused imports: `bool_as_bytes`, `pubkey_as_bytes`, `state::discriminator as DISCRIMINATOR`, and `u64_as_bytes`
// related to macro usage

#[derive(AccountTryFrom, AccountMetas)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub config: &'a AccountInfo,

    #[account(writable)]
    pub admin_rotation_state: &'a AccountInfo,
}

#[test_ser(Discriminator::UpdateConfig)]
#[derive(OptionFlag)]
#[p_serde]
pub struct InstructionData {
    #[optional(
        admin,
        is_paused,
        rotation_timeout,
        registration_fee_amount,
        data_size_range
    )]
    pub flags: BitField,
    pub admin: Pubkey,
    pub is_paused: BitField,
    pub rotation_timeout: Uint32,
    pub registration_fee_amount: Uint64,
    pub data_size_range: Range,
}

// impl InstructionData {
//     const ADMIN: u8 = 0;
//     const IS_PAUSED: u8 = 1;
//     const ROTATION_TIMEOUT: u8 = 2;
//     const REGISTRATION_FEE_AMOUNT: u8 = 3;
//     const DATA_SIZE_RANGE: u8 = 4;

//     #[inline]
//     pub fn get_admin_flag(&self) -> bool {
//         self.flags.get_flag(Self::ADMIN)
//     }

//     #[inline]
//     pub fn set_admin_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::ADMIN, x);
//     }

//     #[inline]
//     pub fn get_is_paused_flag(&self) -> bool {
//         self.flags.get_flag(Self::IS_PAUSED)
//     }

//     #[inline]
//     pub fn set_is_paused_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::IS_PAUSED, x);
//     }

//     #[inline]
//     pub fn get_rotation_timeout_flag(&self) -> bool {
//         self.flags.get_flag(Self::ROTATION_TIMEOUT)
//     }

//     #[inline]
//     pub fn set_rotation_timeout_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::ROTATION_TIMEOUT, x);
//     }

//     #[inline]
//     pub fn get_registration_fee_amount_flag(&self) -> bool {
//         self.flags.get_flag(Self::REGISTRATION_FEE_AMOUNT)
//     }

//     #[inline]
//     pub fn set_registration_fee_amount_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::REGISTRATION_FEE_AMOUNT, x);
//     }

//     #[inline]
//     pub fn get_data_size_range_flag(&self) -> bool {
//         self.flags.get_flag(Self::DATA_SIZE_RANGE)
//     }

//     #[inline]
//     pub fn set_data_size_range_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::DATA_SIZE_RANGE, x);
//     }
// }
