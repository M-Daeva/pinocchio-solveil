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
                _ => Err(ProgramError::InvalidInstructionData)?,
            };

            instruction(accounts, data)
        }
    }
}
