use {
    crate::state::Discriminator,
    base::{traits::DataLen, types::Result},
    bytemuck::{Pod, Zeroable},
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountTryFrom, AccountMetas)]
#[repr(C)]
pub struct Accounts<'a> {
    pub system_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub user_id_pre: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

#[test_ser(Discriminator::ConfirmAccountRotation)]
#[p_serde]
pub struct InstructionData {}
