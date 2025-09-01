use {
    base::{
        accounts::{
            AccountCheck, AccountClose, ProgramAccount, ProgramAccountCheck, SignerAccount,
            SystemProgram,
        },
        types::{AccountData, ZeroCopyDeserialize},
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, UserId},
        types::close_account::{Accounts, InstructionData},
    },
};

pub fn close_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let InstructionData {} = InstructionData::deserialize_from(instruction_data, 0)?.0;

    let Accounts {
        system_program,
        sender,
        user_id,
        user_account,
        user_rotation_state,
    } = Accounts::try_from(accounts)?;

    SystemProgram::check(system_program)?;
    SignerAccount::check(sender)?;
    ProgramAccount::check::<UserId>(user_id, &crate::ID, Some(&[SEED::USER_ID, sender.key()]))?;
    // user_account, // TODO: should check
    // user_rotation_state,

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
