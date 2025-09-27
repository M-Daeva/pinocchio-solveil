use {
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        converters::deserialize,
        error::AuthError,
        helpers::get_clock_time,
        types::StorageW,
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{seed as SEED, Config, RotationState},
        types::update_config::{Accounts, InstructionData},
    },
};

pub fn update_config(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let ix: &InstructionData = deserialize(instruction_data)?;
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

    // === load storages ===

    let mut config = StorageW::<Config>::load(config)?;

    // === use guards ===

    // check sender
    if sender.key() != &config.admin {
        Err(AnyError::Auth(AuthError::Unauthorized))?;
    }

    if let Some(admin) = ix.get_admin() {
        if &admin == sender.key() {
            Err(AnyError::Auth(AuthError::UselessRotation))?;
        }

        StorageW::<RotationState>::update(admin_rotation_state, |x| {
            x.new_owner = admin;
            x.expiration_date
                .set(get_clock_time()? + config.rotation_timeout.get() as u64);
            Ok(())
        })?;
    }

    if let Some(x) = ix.get_is_paused() {
        config.is_paused = x;
    }

    if let Some(x) = ix.get_rotation_timeout() {
        config.rotation_timeout = x;
    }

    if let Some(x) = ix.get_registration_fee_amount() {
        config.registration_fee.amount = x;
    }

    if let Some(x) = ix.get_data_size_range() {
        config.data_size_range = x;
    }

    Ok(())
}
