use {
    crate::state::{Discriminator, ACCOUNT_DATA_SIZE_MAX},
    base::types::Result,
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_serialize,
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

#[test_serialize(Discriminator::WriteData)]
#[derive(Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct InstructionData {
    pub data: [u8; ACCOUNT_DATA_SIZE_MAX as usize],
    pub nonce: [u8; 8],
}

impl Default for InstructionData {
    fn default() -> Self {
        Self {
            data: [0u8; ACCOUNT_DATA_SIZE_MAX as usize],
            nonce: [0u8; 8],
        }
    }
}

// /// for tests
// #[cfg(feature = "dev")]
// impl base::types::InstructionSerialize for InstructionData {
//     fn serialize(&self) -> Result<Vec<u8>> {
//         Ok([&[Discriminator::WriteData as u8], bytemuck::bytes_of(self)].concat())
//     }
// }
