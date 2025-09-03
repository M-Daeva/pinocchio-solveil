use {
    crate::state::Discriminator,
    base::types::{Result, Uint32},
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_serialize,
    macro_try_from::AccountTryFrom,
    macro_zc_serde::p_serde,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountTryFrom, AccountMetas)]
#[repr(C)]
pub struct Accounts<'a> {
    pub system_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

#[test_serialize(Discriminator::ReopenAccount)]
#[p_serde]
pub struct InstructionData {
    pub max_data_size: Uint32,
}
