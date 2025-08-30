use {
    base::{
        accounts::{AccountClose, ProgramAccount},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::UserId,
        types::close_account::{Accounts, InstructionData},
    },
};

pub fn close_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        sender,
        user_id,
        user_account,
        user_rotation_state,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData {} = InstructionData::try_from(instruction_data)?;

    // === load storages ===

    let mut user_id_storage = AccountData::<UserId>::init(user_id)?;
    let mut user_id = user_id_storage.load()?;

    // === use guards ===

    // only open account can be closed
    if !user_id.is_open {
        Err(AnyError::Custom(CustomError::AccountIsNotOpened))?;
    }

    // === save storages ===

    user_id.is_open = false;
    user_id_storage.save(user_id)?;

    // === close accounts ===

    ProgramAccount::close(user_account, sender)?;
    ProgramAccount::close(user_rotation_state, sender)?;

    Ok(())
}
