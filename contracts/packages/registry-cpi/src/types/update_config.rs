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
