use {
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        converters::deserialize,
        types::{StorageR, StorageW},
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, UserAccount, UserId},
        types::write_data::{Accounts, InstructionData},
    },
};

pub fn write_data(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let InstructionData { data, nonce } = deserialize(instruction_data)?;

    let Accounts {
        sender,
        user_id,
        user_account,
    } = Accounts::try_from(accounts)?;

    SignerAccount::check(sender)?;
    ProgramAccount::check::<UserId>(user_id, &crate::ID, Some(&[SEED::USER_ID, sender.key()]))?;
    // user_account, // TODO: should check

    // === load storages ===

    let user_id_storage = StorageR::<UserId>::init(user_id)?;
    let user_id = user_id_storage.load()?;

    let mut user_account_storage = StorageW::<UserAccount>::init(user_account)?;
    let user_account = user_account_storage.load()?;

    // === use guards ===

    if !user_id.get_is_activated_flag() {
        Err(AnyError::Custom(CustomError::AccountIsNotActivated))?;
    }

    if data.len() > user_account.max_size.get() as usize {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsExceeded))?;
    }

    if nonce == &user_account.nonce {
        Err(AnyError::Custom(CustomError::BadNonce))?;
    }

    // === save storages ===

    user_account.data = *data;
    user_account.nonce = *nonce;

    Ok(())
}
