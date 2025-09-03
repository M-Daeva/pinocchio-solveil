use {
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        converters::deserialize,
        error::AuthError,
        helpers::get_clock_time,
        types::Storage,
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

    let mut config_storage = Storage::<Config>::init(config)?;
    let config = config_storage.load_mut()?;

    // === use guards ===

    // check sender
    if sender.key() != &config.admin {
        Err(AnyError::Auth(AuthError::Unauthorized))?;
    }

    if ix.get_admin_flag() {
        if &ix.admin == sender.key() {
            Err(AnyError::Auth(AuthError::UselessRotation))?;
        }

        Storage::<RotationState>::init(admin_rotation_state)?.update(|x| {
            x.new_owner = ix.admin;
            x.expiration_date
                .set(get_clock_time()? + config.rotation_timeout.get() as u64);
            Ok(())
        })?;
    }

    if ix.get_is_paused_flag() {
        config.is_paused = ix.is_paused;
    }

    if ix.get_rotation_timeout_flag() {
        config.rotation_timeout = ix.rotation_timeout;
    }

    if ix.get_registration_fee_amount_flag() {
        config.registration_fee.amount = ix.registration_fee_amount;
    }

    if ix.get_data_size_range_flag() {
        config.data_size_range = ix.data_size_range;
    }

    Ok(())
}
