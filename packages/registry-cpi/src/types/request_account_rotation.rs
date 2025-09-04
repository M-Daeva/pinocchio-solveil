use {
    crate::state::Discriminator,
    base::{traits::DataLen, types::Result},
    bytemuck::{Pod, Zeroable},
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
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

#[test_ser(Discriminator::RequestAccountRotation)]
#[p_serde]
pub struct InstructionData {
    pub new_owner: Pubkey,
}
