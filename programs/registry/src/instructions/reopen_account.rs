use {
    base::{
        accounts::{ProgramAccount, ProgramAccountInit},
        helpers::{create_account_with_signer, get_clock_time},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Config, RotationState, UserAccount, UserId},
        types::reopen_account::{Accounts, InstructionData},
    },
};

pub fn reopen_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        sender,
        config,
        user_id,
        user_account,
        user_rotation_state,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData { max_data_size } = InstructionData::try_from(instruction_data)?;

    // get storages
    //
    let config = AccountData::<Config>::init(config)?.load()?;

    let mut user_id_storage = AccountData::<UserId>::init(user_id)?;
    let mut user_id = user_id_storage.load()?;

    // only closed account can be open
    if user_id.is_open {
        Err(AnyError::Custom(CustomError::OpenAccountTwice))?;
    }

    // validate max allocated data size
    if max_data_size < config.data_size_range.min || max_data_size > config.data_size_range.max {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsOutOfRange))?;
    }

    let user_seed_id = &user_id.id.to_le_bytes();

    // create and write pda
    //
    let bump_ref = &[user_id.account_bump];
    let signer_seeds = &seeds!(SEED::USER_ACCOUNT, user_seed_id, bump_ref);
    let space = UserAccount::get_space(max_data_size) as u64;
    create_account_with_signer(sender, user_account, space, signer_seeds, &crate::ID)?;
    AccountData::init(user_account)?.save(UserAccount {
        data: String::default(),
        nonce: 0,
        max_size: max_data_size,
    })?;

    let bump_ref = &[user_id.rotation_state_bump];
    let signer_seeds = &seeds!(SEED::USER_ROTATION_STATE, user_seed_id, bump_ref);
    ProgramAccount::init::<RotationState>(sender, user_rotation_state, signer_seeds, &crate::ID)?;
    AccountData::init(user_rotation_state)?.save(RotationState {
        owner: *sender.key(),
        new_owner: None,
        expiration_date: get_clock_time()?,
    })?;

    // update storages
    //
    user_id.is_open = true;
    user_id_storage.save(user_id)?;

    Ok(())
}
