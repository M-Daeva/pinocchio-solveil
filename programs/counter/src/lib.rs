#![allow(unexpected_cfgs)]

use {
    counter_cpi::{state::discriminator as DISCRIMINATOR, ID},
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
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
                DISCRIMINATOR::SET => instructions::set,
                _ => Err(ProgramError::InvalidInstructionData)?,
            };

            instruction(accounts, data)
        }
    }
}
