use {
    crate::state::Discriminator,
    base::{
        helpers::{get_flag, set_flag},
        types::Result,
    },
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
    pub token_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    // handle the option on client
    #[account(writable)]
    pub recipient: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    pub revenue_mint: &'a AccountInfo,

    #[account(writable)]
    pub revenue_recipient_ata: &'a AccountInfo,

    #[account(writable)]
    pub revenue_app_ata: &'a AccountInfo,
}

#[test_serialize(Discriminator::WithdrawRevenue)]
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct InstructionData {
    flags: u8,
    amount: [u8; 8],
}

impl InstructionData {
    const AMOUNT: u8 = 0;

    #[inline]
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline]
    pub fn set_amount(&mut self, value: u64) {
        self.amount = value.to_le_bytes();
    }

    #[inline]
    pub fn is_amount(&self) -> bool {
        get_flag(self.flags, Self::AMOUNT)
    }

    #[inline]
    pub fn set_is_amount(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::AMOUNT, flag);
    }
}
