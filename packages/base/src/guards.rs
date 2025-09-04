use {
    crate::constants::{
        TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET, TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR,
    },
    pinocchio::{
        account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
    },
};

// TODO: use custom guard errors for simpler debugging

#[inline]
pub fn check_signer(sender: &AccountInfo) -> ProgramResult {
    if !sender.is_signer() {
        Err(ProgramError::MissingRequiredSignature)?;
    }

    Ok(())
}

#[inline]
pub fn check_account_owner(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    if !account.is_owned_by(program_id) {
        Err(ProgramError::InvalidAccountOwner)?;
    }

    Ok(())
}

#[inline]
pub fn check_account_data_len(account: &AccountInfo, len: usize) -> ProgramResult {
    if account.data_len() != len {
        Err(ProgramError::InvalidAccountData)?;
    }

    Ok(())
}

#[inline]
pub fn check_token_2022_data_len(account: &AccountInfo, len: usize) -> ProgramResult {
    let data = account.try_borrow_data()?;

    if data.len() != len {
        if data.len() <= TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET {
            Err(ProgramError::InvalidAccountData)?;
        }

        if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET] != TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR {
            Err(ProgramError::InvalidAccountData)?;
        }
    }

    Ok(())
}

#[inline]
pub fn check_system_program(system_program: &AccountInfo) -> ProgramResult {
    if system_program.key() != &pinocchio_system::ID {
        Err(ProgramError::IncorrectProgramId)?;
    }

    Ok(())
}

#[inline]
pub fn check_ix_data_len(data: &[u8], end_index: usize) -> ProgramResult {
    if data.len() < end_index {
        Err(ProgramError::InvalidInstructionData)?;
    }

    Ok(())
}

#[inline]
pub fn check_derived_pda(pda: &Pubkey, account: &AccountInfo) -> ProgramResult {
    if pda != account.key() {
        Err(ProgramError::InvalidSeeds)?;
    }

    Ok(())
}
