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
    codama::CodamaInstruction,
    macro_optional_flag::OptionFlag,
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

#[derive(CodamaInstruction, OptionFlag)]
// #[codama(name = "Init")]
#[test_ser(Discriminator::Init)]
#[p_serde]
pub struct InstructionData {
    #[optional(rotation_timeout, account_registration_fee, account_data_size_range)]
    pub flags: BitField,
    pub rotation_timeout: Uint32,
    pub account_registration_fee: AssetItem,
    pub account_data_size_range: Range,
}
