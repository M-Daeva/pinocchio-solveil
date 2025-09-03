use {
    crate::state::Discriminator,
    base::types::Result,
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_serialize,
    macro_try_from::AccountTryFrom,
    macro_zc_serde::p_serde,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

#[derive(AccountTryFrom, AccountMetas)]
#[repr(C)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

#[test_serialize(Discriminator::RequestAccountRotation)]
#[p_serde]
pub struct InstructionData {
    pub new_owner: Pubkey,
}
