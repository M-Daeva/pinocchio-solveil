use {
    crate::state::Discriminator,
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
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct InstructionData {
    max_data_size: [u8; 4],
}

impl InstructionData {
    #[inline]
    pub fn max_data_size(&self) -> u32 {
        u32::from_le_bytes(self.max_data_size)
    }

    #[inline]
    pub fn set_max_data_size(&mut self, value: u32) {
        self.max_data_size = value.to_le_bytes();
    }
}
