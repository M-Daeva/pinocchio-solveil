use {
    base::{
        accounts::{
            AccountCheck, ProgramAccount, ProgramAccountCheck, ProgramAccountInit, SignerAccount,
            SystemProgram,
        },
        converters::deserialize,
        helpers::{create_account_with_signer, get_and_check_pda, get_clock_time},
        types::Storage,
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Bump, Config, RotationState, UserAccount, UserCounter, UserId},
        types::create_account::{Accounts, InstructionData},
    },
};

pub fn create_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let ix: &InstructionData = deserialize(instruction_data)?;
    let Accounts {
        system_program,
        sender,
        bump,
        config,
        user_counter,
        user_id,
        user_account,
        user_rotation_state,
    } = Accounts::try_from(accounts)?;

    SystemProgram::check(system_program)?;
    SignerAccount::check(sender)?;
    ProgramAccount::check::<Bump>(bump, &crate::ID, Some(&[SEED::BUMP]))?;
    ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
    ProgramAccount::check::<UserCounter>(user_counter, &crate::ID, Some(&[SEED::USER_COUNTER]))?;
    // user_id,
    // user_account,
    // user_rotation_state,

    // === load storages ===

    let config = Storage::<Config>::init(config)?.load()?;

    // let mut user_counter_storage = Storage::<UserCounter>::init(user_counter)?;
    // let mut user_counter = user_counter_storage.load()?;

    // let mut data = user_counter.try_borrow_mut_data()?;
    // let mut counter = try_from_bytes_mut::<UserCounter>(&mut data)
    //     .map_err(|_| ProgramError::InvalidStorage)?;

    let mut user_counter_storage = Storage::<UserCounter>::init(user_counter)?;
    let user_counter = user_counter_storage.load()?;

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
        UserAccount::get_space(max_data_size),
        &seeds!(SEED::USER_ACCOUNT, user_seed_id, &[user_account_bump]),
        &crate::ID,
    )?;
    let mut user_account_storage = Storage::init(user_account)?;
    let mut user_account = UserAccount::default();
    user_account.set_max_size(max_data_size);
    user_account_storage.save(&mut user_account)?;

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
    Storage::init(user_rotation_state)?.save(RotationState {
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
    Storage::init(user_id)?.save(UserId {
        id: current_user_id,
        is_open: true,
        is_activated: false,
        account_bump: user_account_bump,
        rotation_state_bump: user_rotation_state_bump,
    })?;

    // === save storages ===

    // user_counter.last_user_id = current_user_id;
    // user_counter_storage.save(user_counter)?;

    user_counter_storage.save(&mut UserCounter {
        last_user_id: current_user_id,
    })?;

    Ok(())
}
