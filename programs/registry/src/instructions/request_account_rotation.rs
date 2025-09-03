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
        state::{seed as SEED, Bump, Config, RotationState, UserId},
        types::request_account_rotation::{Accounts, InstructionData},
    },
};

pub fn request_account_rotation(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix: &InstructionData = deserialize(instruction_data)?;
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

    let config_storage = Storage::<Config>::init(config)?;
    let config = config_storage.load()?;

    Storage::<RotationState>::init(user_rotation_state)?.update(|x| {
        // === use guards ===

        if &ix.new_owner == sender.key() {
            Err(AnyError::Auth(AuthError::UselessRotation))?;
        }

        // === save storages ===

        x.new_owner = ix.new_owner;
        x.set_expiration_date(get_clock_time()? + config.rotation_timeout() as u64);
        Ok(())
    })
}
