use {
    crate::state::Discriminator,
    base::{
        traits::DataLen,
        types::{Result, Uint32},
    },
    bytemuck::{Pod, Zeroable},
    codama::CodamaInstruction,
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountTryFrom, AccountMetas)]
pub struct Accounts<'a> {
    pub system_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    #[account(writable)]
    pub user_counter: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

#[derive(CodamaInstruction)]
// #[codama(name = "CreateAccount")]
#[test_ser(Discriminator::CreateAccount)]
#[p_serde]
pub struct InstructionData {
    pub max_data_size: Uint32,
}
