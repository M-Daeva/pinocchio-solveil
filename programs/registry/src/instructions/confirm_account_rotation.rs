use {
    base::{
        accounts::{AccountClose, ProgramAccount, ProgramAccountInit},
        error::AuthError,
        helpers::{get_and_check_pda, get_clock_time},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{seed as SEED, RotationState, UserId},
        types::confirm_account_rotation::{Accounts, InstructionData},
    },
};

pub fn confirm_account_rotation(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let Accounts {
        sender,
        user_id_pre: user_id_pre_acc,
        user_id,
        user_rotation_state,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData {} = InstructionData::try_from(instruction_data)?;

    // create account for new user
    let user_id_seeds = &[SEED::USER_ID, sender.key()];
    let (_user_id_pda, user_id_bump) = get_and_check_pda(user_id_seeds, &crate::ID, Some(user_id))?;

    let bump_ref = &[user_id_bump];
    let signer_seeds = &seeds!(SEED::USER_ID, sender.key(), bump_ref);
    ProgramAccount::init::<UserId>(sender, user_id, signer_seeds, &crate::ID)?;
    let mut user_id_storage = AccountData::<UserId>::init(user_id)?;

    // get storages
    //
    let user_id_pre = AccountData::<UserId>::init(user_id_pre_acc)?.load()?;

    let mut user_rotation_state_storage = AccountData::<RotationState>::init(user_rotation_state)?;
    let user_rotation_state = user_rotation_state_storage.load()?;

    let clock_time = get_clock_time()?;

    match user_rotation_state.new_owner {
        None => Err(AnyError::Auth(AuthError::NoNewOwner))?,
        Some(new_owner) => {
            if sender.key() != &new_owner {
                Err(AnyError::Auth(AuthError::Unauthorized))?;
            }

            if clock_time >= user_rotation_state.expiration_date {
                Err(AnyError::Auth(AuthError::TransferOwnerDeadline))?;
            }

            user_id_storage.save(user_id_pre)?;

            user_rotation_state_storage.save(RotationState {
                owner: new_owner,
                new_owner: None,
                expiration_date: clock_time,
            })?;

            ProgramAccount::close(user_id_pre_acc, sender)?;
        }
    }

    Ok(())
}
