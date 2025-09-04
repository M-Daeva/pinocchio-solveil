use {
    base::{error::AuthError, helpers::get_clock_time, types::StorageW},
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{Config, RotationState},
        types::confirm_admin_rotation::Accounts,
    },
};

pub fn confirm_admin_rotation(accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    // let ix: &InstructionData = deserialize(instruction_data)?;
    let Accounts {
        sender,
        config,
        admin_rotation_state,
    } = Accounts::try_from(accounts)?;

    StorageW::<RotationState>::update(admin_rotation_state, |admin_rotation_state| {
        // === use guards ===

        if admin_rotation_state.new_owner == admin_rotation_state.owner {
            Err(AnyError::Auth(AuthError::NoNewOwner))?;
        }

        if sender.key() != &admin_rotation_state.new_owner {
            Err(AnyError::Auth(AuthError::Unauthorized))?;
        }

        let clock_time = get_clock_time()?;

        if clock_time >= admin_rotation_state.expiration_date.get() {
            Err(AnyError::Auth(AuthError::TransferOwnerDeadline))?;
        }

        // === save storages ===

        admin_rotation_state.owner = admin_rotation_state.new_owner;
        admin_rotation_state.expiration_date.set(clock_time);

        StorageW::<Config>::update(config, |x| {
            x.admin = admin_rotation_state.new_owner;
            Ok(())
        })
    })
}
