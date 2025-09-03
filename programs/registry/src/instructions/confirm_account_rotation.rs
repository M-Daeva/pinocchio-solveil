use {
    base::{
        accounts::{
            AccountCheck, AccountClose, ProgramAccount, ProgramAccountInit, SignerAccount,
            SystemProgram,
        },
        error::AuthError,
        helpers::{get_and_check_pda, get_clock_time},
        types::Storage,
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{seed as SEED, RotationState, UserId},
        types::confirm_account_rotation::Accounts,
    },
};

pub fn confirm_account_rotation(
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // let ix: &InstructionData = deserialize(instruction_data)?;
    let Accounts {
        system_program,
        sender,
        user_id_pre,
        user_id,
        user_rotation_state,
    } = Accounts::try_from(accounts)?;

    SystemProgram::check(system_program)?;
    SignerAccount::check(sender)?;
    // user_id_pre,
    // ProgramAccount::check(
    //     user_id,
    //     &crate::ID,
    //     UserId::get_space(),
    //     Some(&[SEED::USER_ID, sender.key()]),
    // )?;
    // user_rotation_state, // TODO: should check

    // === init and write pda ===

    // new user doesn't have account yet, so it must be created
    let (_, user_id_bump) =
        get_and_check_pda(&[SEED::USER_ID, sender.key()], &crate::ID, Some(user_id))?;
    ProgramAccount::init::<UserId>(
        sender,
        user_id,
        &seeds!(SEED::USER_ID, sender.key(), &[user_id_bump]),
        &crate::ID,
    )?;

    Storage::<RotationState>::init(user_rotation_state)?.update(|user_rotation_state| {
        // === use guards ===

        if user_rotation_state.new_owner == user_rotation_state.owner {
            Err(AnyError::Auth(AuthError::NoNewOwner))?;
        }

        if sender.key() != &user_rotation_state.new_owner {
            Err(AnyError::Auth(AuthError::Unauthorized))?;
        }

        let clock_time = get_clock_time()?;

        if clock_time >= user_rotation_state.expiration_date.get() {
            Err(AnyError::Auth(AuthError::TransferOwnerDeadline))?;
        }

        // === save storages ===

        user_rotation_state.owner = user_rotation_state.new_owner;
        user_rotation_state.expiration_date.set(clock_time);

        Storage::<UserId>::init(user_id)?.update(|x| {
            *x = *Storage::init(user_id_pre)?.load_mut()?;
            Ok(())
        })
    })?;

    // === close accounts ===

    ProgramAccount::close(user_id_pre, sender)
}
