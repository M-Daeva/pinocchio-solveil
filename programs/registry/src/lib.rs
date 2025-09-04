#![no_std]
#![allow(unexpected_cfgs)]

use {
    core::mem::transmute,
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
    registry_cpi::{state::Discriminator as D, ID},
};

mod instructions;
use instructions as i;

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
                D::Init => i::init,
                D::UpdateConfig => i::update_config,
                D::ConfirmAdminRotation => i::confirm_admin_rotation,
                D::WithdrawRevenue => i::withdraw_revenue,
                // creates user PDA account taking rent exempt in SOL
                D::CreateAccount => i::create_account,
                // 1st step to to change allocated data space or just to redeem rent
                D::CloseAccount => i::close_account,
                // 2nd step to to change allocated data space
                D::ReopenAccount => i::reopen_account,
                // activates account with fee asset payment
                D::ActivateAccount => i::activate_account,
                D::WriteData => i::write_data,
                D::RequestAccountRotation => i::request_account_rotation,
                // updates address - id pair
                D::ConfirmAccountRotation => i::confirm_account_rotation,
            };

            instruction(accounts, data)
        }
    }
}
