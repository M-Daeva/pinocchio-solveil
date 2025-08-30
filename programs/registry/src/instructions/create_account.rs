use {
    base::{
        accounts::{ProgramAccount, ProgramAccountInit},
        helpers::{create_account_with_signer, get_and_check_pda, get_clock_time},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Config, RotationState, UserAccount, UserCounter, UserId},
        types::create_account::{Accounts, InstructionData},
    },
};

pub fn create_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        sender,
        config,
        user_counter,
        user_id,
        user_account,
        user_rotation_state,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData { max_data_size } = InstructionData::try_from(instruction_data)?;

    // === load storages ===

    let config = AccountData::<Config>::init(config)?.load()?;

    let mut user_counter_storage = AccountData::<UserCounter>::init(user_counter)?;
    let mut user_counter = user_counter_storage.load()?;

    // === use guards ===

    // don't allow register accounts in paused program
    if config.is_paused {
        Err(AnyError::Custom(CustomError::ContractIsPaused))?;
    }

    // validate max allocated data size
    if max_data_size < config.data_size_range.min || max_data_size > config.data_size_range.max {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsOutOfRange))?;
    }

    // === core logic ===

    let current_user_id = user_counter.last_user_id + 1;
    let user_seed_id = &current_user_id.to_le_bytes();

    // === init and write pda ===

    // user_account
    let (_, user_account_bump) = get_and_check_pda(
        &[SEED::USER_ACCOUNT, user_seed_id],
        &crate::ID,
        Some(user_account),
    )?;
    create_account_with_signer(
        sender,
        user_account,
        UserAccount::get_space(max_data_size) as u64,
        &seeds!(SEED::USER_ACCOUNT, user_seed_id, &[user_account_bump]),
        &crate::ID,
    )?;
    AccountData::init(user_account)?.save(UserAccount {
        data: String::default(),
        nonce: 0,
        max_size: max_data_size,
    })?;

    // user_rotation_state
    let (_, user_rotation_state_bump) = get_and_check_pda(
        &[SEED::USER_ROTATION_STATE, user_seed_id],
        &crate::ID,
        Some(user_rotation_state),
    )?;
    ProgramAccount::init::<RotationState>(
        sender,
        user_rotation_state,
        &seeds!(
            SEED::USER_ROTATION_STATE,
            user_seed_id,
            &[user_rotation_state_bump]
        ),
        &crate::ID,
    )?;
    AccountData::init(user_rotation_state)?.save(RotationState {
        owner: *sender.key(),
        new_owner: None,
        expiration_date: get_clock_time()?,
    })?;

    // user_id
    let (_, user_id_bump) =
        get_and_check_pda(&[SEED::USER_ID, sender.key()], &crate::ID, Some(user_id))?;
    ProgramAccount::init::<UserId>(
        sender,
        user_id,
        &seeds!(SEED::USER_ID, sender.key(), &[user_id_bump]),
        &crate::ID,
    )?;
    AccountData::init(user_id)?.save(UserId {
        id: current_user_id,
        is_open: true,
        is_activated: false,
        account_bump: user_account_bump,
        rotation_state_bump: user_rotation_state_bump,
    })?;

    // === save storages ===

    user_counter.last_user_id = current_user_id;
    user_counter_storage.save(user_counter)?;

    Ok(())
}
