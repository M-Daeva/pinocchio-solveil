use {
    base::{
        accounts::{
            AccountCheck, ProgramAccount, ProgramAccountCheck, ProgramAccountInit, SignerAccount,
            SystemProgram,
        },
        converters::deserialize,
        helpers::{create_account_with_signer, get_and_check_pda, get_clock_time},
        types::{StorageR, StorageW},
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

    let config = StorageR::<Config>::load(config)?;

    let mut user_counter_storage = StorageW::<UserCounter>::init(user_counter)?;
    let user_counter = user_counter_storage.load()?;
    user_counter
        .last_user_id
        .set(user_counter.last_user_id.get() + 1);

    let user_seed_id = &user_counter.last_user_id.get_raw();

    // === use guards ===

    // don't allow register accounts in paused program
    if config.is_paused.get_bit() {
        Err(AnyError::Custom(CustomError::ContractIsPaused))?;
    }

    let max_data_size = ix.max_data_size.get();

    // validate max allocated data size
    if max_data_size < config.data_size_range.min.get()
        || max_data_size > config.data_size_range.max.get()
    {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsOutOfRange))?;
    }

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
    StorageW::init(user_account)?.update(|x| {
        *x = UserAccount::default();
        x.max_size.set(max_data_size);
        Ok(())
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
    StorageW::<RotationState>::init(user_rotation_state)?.update(|x| {
        x.owner = *sender.key();
        x.new_owner = *sender.key();
        x.expiration_date.set(get_clock_time()?);
        Ok(())
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
    StorageW::<UserId>::init(user_id)?.update(|x| {
        x.id.set_raw(*user_seed_id);
        x.set_is_open_flag(true);
        x.set_is_activated_flag(false);
        x.account_bump = user_account_bump;
        x.rotation_state_bump = user_rotation_state_bump;
        Ok(())
    })
}
