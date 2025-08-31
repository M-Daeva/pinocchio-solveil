use {
    base::{
        accounts::{
            AccountCheck, ProgramAccount, ProgramAccountCheck, ProgramAccountInit, SignerAccount,
            SystemProgram,
        },
        helpers::{create_account_with_signer, get_clock_time},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Bump, Config, RotationState, UserAccount, UserId},
        types::reopen_account::{Accounts, InstructionData},
    },
};

pub fn reopen_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let InstructionData { max_data_size } = InstructionData::try_from(instruction_data)?;

    let Accounts {
        system_program,
        sender,
        bump,
        config,
        user_id,
        user_account,
        user_rotation_state,
    } = Accounts::try_from(accounts)?;

    SystemProgram::check(system_program)?;
    SignerAccount::check(sender)?;
    ProgramAccount::check::<Bump>(bump, &crate::ID, Some(&[SEED::BUMP]))?;
    ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
    ProgramAccount::check::<UserId>(user_id, &crate::ID, Some(&[SEED::USER_ID, sender.key()]))?;
    // user_account,
    // user_rotation_state,

    // === load storages ===

    let config = AccountData::<Config>::init(config)?.load()?;

    let mut user_id_storage = AccountData::<UserId>::init(user_id)?;
    let mut user_id = user_id_storage.load()?;

    // === use guards ===

    // only closed account can be open
    if user_id.is_open {
        Err(AnyError::Custom(CustomError::OpenAccountTwice))?;
    }

    // validate max allocated data size
    if max_data_size < config.data_size_range.min || max_data_size > config.data_size_range.max {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsOutOfRange))?;
    }

    // === init and write pda ===

    let user_seed_id = &user_id.id.to_le_bytes();

    // user_account
    create_account_with_signer(
        sender,
        user_account,
        UserAccount::get_space(max_data_size),
        &seeds!(SEED::USER_ACCOUNT, user_seed_id, &[user_id.account_bump]),
        &crate::ID,
    )?;
    AccountData::init(user_account)?.save(UserAccount {
        data: String::default(),
        nonce: 0,
        max_size: max_data_size,
    })?;

    // user_rotation_state
    ProgramAccount::init::<RotationState>(
        sender,
        user_rotation_state,
        &seeds!(
            SEED::USER_ROTATION_STATE,
            user_seed_id,
            &[user_id.rotation_state_bump]
        ),
        &crate::ID,
    )?;
    AccountData::init(user_rotation_state)?.save(RotationState {
        owner: *sender.key(),
        new_owner: None,
        expiration_date: get_clock_time()?,
    })?;

    // === save storages ===

    user_id.is_open = true;
    user_id_storage.save(user_id)?;

    Ok(())
}
