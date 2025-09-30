use {
    crate::state::Discriminator,
    base::{
        traits::DataLen,
        types::{Result, String4096, Uint64},
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
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,
}

#[derive(CodamaInstruction)]
// #[codama(name = "WriteData")]
#[test_ser(Discriminator::WriteData)]
#[p_serde]
pub struct InstructionData {
    pub data: String4096,
    pub nonce: Uint64,
}
