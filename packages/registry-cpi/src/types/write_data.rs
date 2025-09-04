use {
    crate::state::{Discriminator, ACCOUNT_DATA_SIZE_MAX},
    base::types::{Result, Uint64},
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountTryFrom, AccountMetas)]
#[repr(C)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,
}

#[test_ser(Discriminator::WriteData)]
#[derive(Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct InstructionData {
    pub data: [u8; ACCOUNT_DATA_SIZE_MAX as usize],
    pub nonce: Uint64,
}

impl Default for InstructionData {
    fn default() -> Self {
        Self {
            data: [0; ACCOUNT_DATA_SIZE_MAX as usize],
            nonce: Uint64::default(),
        }
    }
}
