use {
    base::{error::AuthError, helpers::get_clock_time, types::AccountData},
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{Config, RotationState},
        types::confirm_admin_rotation::{Accounts, InstructionData},
    },
};

pub fn confirm_admin_rotation(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        sender,
        config,
        admin_rotation_state,
    } = Accounts::try_from(accounts)?;

    let InstructionData {} = InstructionData::try_from(instruction_data)?;

    // === load storages ===

    let mut config_storage = AccountData::<Config>::init(config)?;

    let mut admin_rotation_state_storage =
        AccountData::<RotationState>::init(admin_rotation_state)?;
    let admin_rotation_state = admin_rotation_state_storage.load()?;

    // === use guards ===

    match admin_rotation_state.new_owner {
        None => Err(AnyError::Auth(AuthError::NoNewOwner))?,
        Some(new_admin) => {
            let clock_time = get_clock_time()?;

            if sender.key() != &new_admin {
                Err(AnyError::Auth(AuthError::Unauthorized))?;
            }

            if clock_time >= admin_rotation_state.expiration_date {
                Err(AnyError::Auth(AuthError::TransferOwnerDeadline))?;
            }

            // === save storages ===

            config_storage.update(|mut x| {
                x.admin = new_admin;
                Ok(x)
            })?;

            admin_rotation_state_storage.save(RotationState {
                owner: new_admin,
                new_owner: None,
                expiration_date: clock_time,
            })?;
        }
    }

    Ok(())
}
