use {
    base::{
        accounts::{
            AccountCheck, ProgramAccount, ProgramAccountCheck, ProgramAccountInit, SignerAccount,
            SystemProgram,
        },
        converters::deserialize,
        helpers::{create_account_with_signer, get_clock_time},
        types::{StorageR, StorageW},
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Bump, Config, RotationState, UserAccount, UserId},
        types::reopen_account::{Accounts, InstructionData},
    },
};

pub fn reopen_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let ix: &InstructionData = deserialize(instruction_data)?;
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

    let config_storage = StorageR::<Config>::init(config)?;
    let config = config_storage.load()?;

    let mut user_id_storage = StorageW::<UserId>::init(user_id)?;
    let user_id = user_id_storage.load()?;

    // === use guards ===

    // only closed account can be open
    if user_id.get_is_open_flag() {
        Err(AnyError::Custom(CustomError::OpenAccountTwice))?;
    }

    user_id.set_is_open_flag(true);
    let max_data_size = ix.max_data_size.get();
    let user_seed_id = &user_id.id.get_raw();

    // validate max allocated data size
    if max_data_size < config.data_size_range.min.get()
        || max_data_size > config.data_size_range.max.get()
    {
        Err(AnyError::Custom(CustomError::MaxDataSizeIsOutOfRange))?;
    }

    // === init and write pda ===

    // user_account
    create_account_with_signer(
        sender,
        user_account,
        UserAccount::get_space(max_data_size),
        &seeds!(SEED::USER_ACCOUNT, user_seed_id, &[user_id.account_bump]),
        &crate::ID,
    )?;
    StorageW::init(user_account)?.update(|x| {
        *x = UserAccount::default();
        x.max_size.set(max_data_size);
        Ok(())
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
    StorageW::<RotationState>::init(user_rotation_state)?.update(|x| {
        x.owner = *sender.key();
        x.new_owner = *sender.key();
        x.expiration_date.set(get_clock_time()?);
        Ok(())
    })
}
