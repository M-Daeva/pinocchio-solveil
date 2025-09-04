use {
    base::{
        accounts::{
            AccountCheck, AccountClose, ProgramAccount, ProgramAccountCheck, SignerAccount,
            SystemProgram,
        },
        types::StorageW,
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, UserId},
        types::close_account::Accounts,
    },
};

pub fn close_account(accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    // let ix: &InstructionData = deserialize(instruction_data)?;
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

    // === save storages ===

    StorageW::<UserId>::update(user_id, |user_id| {
        // only open account can be closed
        if !user_id.get_is_open_flag() {
            Err(AnyError::Custom(CustomError::AccountIsNotOpened))?;
        }

        user_id.set_is_open_flag(false);
        Ok(())
    })?;

    // === close accounts ===

    ProgramAccount::close(user_account, sender)?;
    ProgramAccount::close(user_rotation_state, sender)
}
