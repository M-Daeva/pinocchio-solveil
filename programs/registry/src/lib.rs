#![allow(unexpected_cfgs)]

use {
    core::mem::transmute,
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
    registry_cpi::{state::Discriminator, ID},
};

mod instructions;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        None => Err(ProgramError::InvalidInstructionData),
        Some((discriminator, data)) => {
            let instruction = match unsafe { transmute(*discriminator) } {
                Discriminator::Init => instructions::init,
                Discriminator::UpdateConfig => instructions::update_config,
                Discriminator::ConfirmAdminRotation => instructions::confirm_admin_rotation,
                Discriminator::WithdrawRevenue => instructions::withdraw_revenue,
                // creates user PDA account taking rent exempt in SOL
                Discriminator::CreateAccount => instructions::create_account,
                // 1st step to to change allocated data space or just to redeem rent
                Discriminator::CloseAccount => instructions::close_account,
                // 2nd step to to change allocated data space
                Discriminator::ReopenAccount => instructions::reopen_account,
                // activates account with fee asset payment
                Discriminator::ActivateAccount => instructions::activate_account,
                Discriminator::WriteData => instructions::write_data,
                Discriminator::RequestAccountRotation => instructions::request_account_rotation,
                // updates address - id pair
                Discriminator::ConfirmAccountRotation => instructions::confirm_account_rotation,
            };

            instruction(accounts, data)
        }
    }
}
