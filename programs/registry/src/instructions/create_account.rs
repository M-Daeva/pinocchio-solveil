use {
    base::{
        accounts::{ProgramAccount, ProgramAccountInit},
        helpers::{get_and_check_pda, get_clock_time},
        types::{AccountData, Space},
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
        bump,
        config,
        user_counter,
        user_id,
        user_account,
        user_rotation_state,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData { max_data_size } = InstructionData::try_from(instruction_data)?;

    // get storages
    //
    let config = AccountData::<Config>::init(config)?.load()?;
    let mut user_counter_storage = AccountData::<UserCounter>::init(user_counter)?;
    let mut user_counter = user_counter_storage.load()?;

    // don't allow register accounts in paused program
    if config.is_paused {
        Err(AnyError::Custom(CustomError::ContractIsPaused))?;
    }

    // validate max allocated data size
    if max_data_size < config.data_size_range.min || max_data_size > config.data_size_range.max {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsOutOfRange))?;
    }

    let current_user_id = user_counter.last_user_id + 1;
    user_counter.last_user_id = current_user_id;

    // get bumps
    //
    let user_id_seeds = &[SEED::USER_ID, sender.key()];
    let (_user_id_pda, user_id_bump) = get_and_check_pda(user_id_seeds, &crate::ID, Some(user_id))?;

    let user_account_seeds = &[
        SEED::USER_ACCOUNT,
        &(user_counter.last_user_id + 1).to_le_bytes(),
    ];
    let (_user_account_pda, user_account_bump) =
        get_and_check_pda(user_account_seeds, &crate::ID, Some(user_account))?;

    let user_rotation_state_seeds = &[
        SEED::USER_ROTATION_STATE,
        &(user_counter.last_user_id + 1).to_le_bytes(),
    ];
    let (_user_rotation_state_pda, user_rotation_state_bump) = get_and_check_pda(
        user_rotation_state_seeds,
        &crate::ID,
        Some(user_rotation_state),
    )?;

    // create and write pda
    //
    let bump_ref = &[user_id_bump];
    let signer_seeds = &seeds!(SEED::USER_ID, bump_ref);
    ProgramAccount::init(
        sender,
        user_id,
        UserId::get_space(),
        signer_seeds,
        &crate::ID,
    )?;
    AccountData::init(bump)?.save(UserId {
        id: current_user_id,
        is_open: true,
        is_activated: false,
        account_bump: user_account_bump,
        rotation_state_bump: user_rotation_state_bump,
    })?;

    let bump_ref = &[user_account_bump];
    let signer_seeds = &seeds!(SEED::USER_ACCOUNT, bump_ref);
    ProgramAccount::init(
        sender,
        user_account,
        UserAccount::get_space(max_data_size) as u64,
        signer_seeds,
        &crate::ID,
    )?;
    AccountData::init(bump)?.save(UserAccount {
        data: String::default(),
        nonce: 0,
        max_size: max_data_size,
    })?;

    let bump_ref = &[user_rotation_state_bump];
    let signer_seeds = &seeds!(SEED::USER_ROTATION_STATE, bump_ref);
    ProgramAccount::init(
        sender,
        user_rotation_state,
        RotationState::get_space(),
        signer_seeds,
        &crate::ID,
    )?;
    AccountData::init(bump)?.save(RotationState {
        owner: *sender.key(),
        new_owner: None,
        expiration_date: get_clock_time()?,
    })?;

    // update storages
    //
    user_counter_storage.save(user_counter)?;

    Ok(())
}
