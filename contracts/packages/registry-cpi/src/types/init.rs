use {
    crate::{
        state::Discriminator,
        types::common::{AssetItem, Range},
    },
    base::{
        traits::DataLen,
        types::{BitField, Result, Uint32},
    },
    bytemuck::{Pod, Zeroable},
    macro_optional_flag::OptionFlag,
    // codama::{
    //     CodamaAccount, CodamaAccounts, CodamaErrors, CodamaInstruction, CodamaInstructions,
    //     CodamaType,
    // },
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

// TODO: implement remaining accounts
#[derive(AccountTryFrom, AccountMetas)]
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

#[test_ser(Discriminator::Init)]
#[derive(OptionFlag)]
#[p_serde]
pub struct InstructionData {
    pub flags: BitField,
    #[optional(flags)]
    pub rotation_timeout: Uint32,
    #[optional(flags)]
    pub account_registration_fee: AssetItem,
    #[optional(flags)]
    pub account_data_size_range: Range,
}

// impl InstructionData {
//     const ROTATION_TIMEOUT: u8 = 0;
//     const ACCOUNT_REGISTRATION_FEE: u8 = 1;
//     const ACCOUNT_DATA_SIZE_RANGE: u8 = 2;

//     #[inline]
//     pub fn get_rotation_timeout_flag(&self) -> bool {
//         self.flags.get_flag(Self::ROTATION_TIMEOUT)
//     }

//     #[inline]
//     pub fn set_rotation_timeout_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::ROTATION_TIMEOUT, x);
//     }

//     #[inline]
//     pub fn get_account_registration_fee_flag(&self) -> bool {
//         self.flags.get_flag(Self::ACCOUNT_REGISTRATION_FEE)
//     }

//     #[inline]
//     pub fn set_account_registration_fee_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::ACCOUNT_REGISTRATION_FEE, x);
//     }

//     #[inline]
//     pub fn get_account_data_size_range_flag(&self) -> bool {
//         self.flags.get_flag(Self::ACCOUNT_DATA_SIZE_RANGE)
//     }

//     #[inline]
//     pub fn set_account_data_size_range_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::ACCOUNT_DATA_SIZE_RANGE, x);
//     }
// }
