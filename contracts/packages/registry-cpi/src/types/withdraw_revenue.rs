use {
    crate::state::Discriminator,
    base::{
        traits::DataLen,
        types::{BitField, Result, Uint64},
    },
    bytemuck::{Pod, Zeroable},
    codama::CodamaInstruction,
    macro_optional_flag::OptionFlag,
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountTryFrom, AccountMetas)]
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

#[derive(CodamaInstruction, OptionFlag)]
// #[codama(name = "WithdrawRevenue")]
#[test_ser(Discriminator::WithdrawRevenue)]
#[p_serde]
pub struct InstructionData {
    #[optional(amount)]
    pub flags: BitField,
    pub amount: Uint64,
}
