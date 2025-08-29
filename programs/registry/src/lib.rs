#![allow(unexpected_cfgs)]

use {
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
    registry_cpi::{state::discriminator as DISCRIMINATOR, ID},
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
            let instruction = match *discriminator {
                DISCRIMINATOR::INIT => instructions::init,
                DISCRIMINATOR::UPDATE_CONFIG => instructions::update_config,
                DISCRIMINATOR::CONFIRM_ADMIN_ROTATION => instructions::confirm_admin_rotation,
                DISCRIMINATOR::WITHDRAW_REVENUE => instructions::withdraw_revenue,
                // creates user PDA account taking rent exempt in SOL
                DISCRIMINATOR::CREATE_ACCOUNT => instructions::create_account,
                // 1st step to to change allocated data space or just to redeem rent
                DISCRIMINATOR::CLOSE_ACCOUNT => todo!(),
                // 2nd step to to change allocated data space
                DISCRIMINATOR::REOPEN_ACCOUNT => todo!(),
                // activates account with fee asset payment
                DISCRIMINATOR::ACTIVATE_ACCOUNT => todo!(),
                DISCRIMINATOR::WRITE_DATA => todo!(),
                DISCRIMINATOR::REQUEST_ACCOUNT_ROTATION => todo!(),
                // updates address - id pair
                DISCRIMINATOR::CONFIRM_ACCOUNT_ROTATION => todo!(),
                _ => Err(ProgramError::InvalidInstructionData)?,
            };

            instruction(accounts, data)
        }
    }
}
