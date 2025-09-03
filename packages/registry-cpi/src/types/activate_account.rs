use {
    crate::state::Discriminator,
    base::types::Result,
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
    pub token_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    pub revenue_mint: &'a AccountInfo,

    #[account(writable)]
    pub revenue_sender_ata: &'a AccountInfo,

    #[account(writable)]
    pub revenue_app_ata: &'a AccountInfo,
}

#[test_serialize(Discriminator::ActivateAccount)]
#[p_serde]
pub struct InstructionData {}
