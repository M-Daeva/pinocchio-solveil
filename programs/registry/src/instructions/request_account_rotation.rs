use {
    base::{error::AuthError, helpers::get_clock_time, types::AccountData},
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{Config, RotationState},
        types::request_account_rotation::{Accounts, InstructionData},
    },
};

pub fn request_account_rotation(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let Accounts {
        sender,
        config,
        user_rotation_state,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData { new_owner } = InstructionData::try_from(instruction_data)?;

    // get storages
    //
    let config = AccountData::<Config>::init(config)?.load()?;

    let mut user_rotation_state_storage = AccountData::<RotationState>::init(user_rotation_state)?;
    let mut user_rotation_state = user_rotation_state_storage.load()?;

    if &new_owner == sender.key() {
        Err(AnyError::Auth(AuthError::UselessRotation))?;
    }

    user_rotation_state.new_owner = Some(new_owner);
    user_rotation_state.expiration_date = get_clock_time()? + config.rotation_timeout as u64;
    user_rotation_state_storage.save(user_rotation_state)?;

    Ok(())
}
