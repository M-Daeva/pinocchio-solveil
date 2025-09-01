use {
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        error::AuthError,
        helpers::get_clock_time,
        types::{AccountData, ZeroCopyDeserialize},
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Config, RotationState},
        types::update_config::{Accounts, InstructionData},
    },
};

pub fn update_config(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let InstructionData {
        admin,
        is_paused,
        rotation_timeout,
        registration_fee_amount,
        data_size_range,
    } = InstructionData::deserialize_from(instruction_data, 0)?.0;

    let Accounts {
        sender,
        config,
        admin_rotation_state,
    } = Accounts::try_from(accounts)?;

    SignerAccount::check(sender)?;
    ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
    ProgramAccount::check::<RotationState>(
        admin_rotation_state,
        &crate::ID,
        Some(&[SEED::ADMIN_ROTATION_STATE]),
    )?;

    let mut is_config_updated = false;
    let mut is_admin_rotation_state_updated = false;

    // === load storages ===

    let mut config_storage = AccountData::<Config>::init(config)?;
    let mut config = config_storage.load()?;

    let mut admin_rotation_state_storage =
        AccountData::<RotationState>::init(admin_rotation_state)?;
    let mut admin_rotation_state = admin_rotation_state_storage.load()?;

    // === use guards ===

    // check sender
    if sender.key() != &config.admin {
        Err(AnyError::Auth(AuthError::Unauthorized))?;
    }

    if let Some(new_admin) = admin {
        if &new_admin == sender.key() {
            Err(AnyError::Auth(AuthError::UselessRotation))?;
        }

        admin_rotation_state.new_owner = Some(new_admin);
        admin_rotation_state.expiration_date = get_clock_time()? + config.rotation_timeout as u64;
        is_admin_rotation_state_updated = true;
    }

    if let Some(x) = is_paused {
        config.is_paused = x;
        is_config_updated = true;
    }

    if let Some(x) = rotation_timeout {
        config.rotation_timeout = x;
        is_config_updated = true;
    }

    if let Some(x) = registration_fee_amount {
        config.registration_fee.amount = x;
        is_config_updated = true;
    }

    if let Some(x) = data_size_range {
        config.data_size_range = x;
        is_config_updated = true;
    }

    // don't allow empty instructions
    if !is_config_updated && !is_admin_rotation_state_updated {
        Err(AnyError::Custom(CustomError::NoParameters))?;
    }

    // === save storages ===

    if is_config_updated {
        config_storage.save(config)?;
    }

    if is_admin_rotation_state_updated {
        admin_rotation_state_storage.save(admin_rotation_state)?;
    }

    Ok(())
}
