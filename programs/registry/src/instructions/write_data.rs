use {
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, UserAccount, UserId},
        types::write_data::{Accounts, InstructionData},
    },
};

pub fn write_data(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let InstructionData { data, nonce } = InstructionData::try_from(instruction_data)?;

    let Accounts {
        sender,
        user_id,
        user_account,
    } = Accounts::try_from(accounts)?;

    SignerAccount::check(sender)?;
    ProgramAccount::check::<UserId>(user_id, &crate::ID, Some(&[SEED::USER_ID, sender.key()]))?;
    // user_account, // TODO: should check

    // === load storages ===

    let user_id = AccountData::<UserId>::init(user_id)?.load()?;

    let mut user_account_storage = AccountData::<UserAccount>::init(user_account)?;
    let mut user_account = user_account_storage.load()?;

    // === use guards ===

    if !user_id.is_activated {
        Err(AnyError::Custom(CustomError::AccountIsNotActivated))?;
    }

    if data.len() > user_account.max_size as usize {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsExceeded))?;
    }

    if nonce == user_account.nonce {
        Err(AnyError::Custom(CustomError::BadNonce))?;
    }

    // === save storages ===

    user_account.data = data;
    user_account.nonce = nonce;
    user_account_storage.save(user_account)?;

    Ok(())
}
