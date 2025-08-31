use {
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        error::AuthError,
        helpers::get_clock_time,
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{seed as SEED, Bump, Config, RotationState, UserId},
        types::request_account_rotation::{Accounts, InstructionData},
    },
};

pub fn request_account_rotation(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let InstructionData { new_owner } = InstructionData::try_from(instruction_data)?;

    let Accounts {
        sender,
        bump,
        config,
        user_id,
        user_rotation_state,
    } = Accounts::try_from(accounts)?;

    SignerAccount::check(sender)?;
    ProgramAccount::check::<Bump>(bump, &crate::ID, Some(&[SEED::BUMP]))?;
    ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
    ProgramAccount::check::<UserId>(user_id, &crate::ID, Some(&[SEED::USER_ID, sender.key()]))?;
    // user_rotation_state, // TODO: should check

    // === load storages ===

    let config = AccountData::<Config>::init(config)?.load()?;

    let mut user_rotation_state_storage = AccountData::<RotationState>::init(user_rotation_state)?;
    let mut user_rotation_state = user_rotation_state_storage.load()?;

    // === use guards ===

    if &new_owner == sender.key() {
        Err(AnyError::Auth(AuthError::UselessRotation))?;
    }

    // === save storages ===

    user_rotation_state.new_owner = Some(new_owner);
    user_rotation_state.expiration_date = get_clock_time()? + config.rotation_timeout as u64;
    user_rotation_state_storage.save(user_rotation_state)?;

    Ok(())
}
