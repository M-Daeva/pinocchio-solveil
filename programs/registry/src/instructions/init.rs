use {
    base::{
        accounts::{
            AssociatedTokenAccount, AssociatedTokenAccountInit, ProgramAccount, ProgramAccountInit,
        },
        error::AuthError,
        helpers::{get_and_check_pda, get_clock_time},
        types::{AccountData, Space},
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::AnyError,
        state::{
            seed as SEED, Bump, Config, RotationState, UserCounter, ACCOUNT_DATA_SIZE_MAX,
            ACCOUNT_DATA_SIZE_MIN, ACCOUNT_REGISTRATION_FEE_AMOUNT, ACCOUNT_REGISTRATION_FEE_ASSET,
            CLOCK_TIME_MIN, MAINNET_ADMIN, ROTATION_TIMEOUT,
        },
        types::{
            common::{AssetItem, Range},
            init::{Accounts, InstructionData},
        },
    },
};

pub fn init(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        system_program,
        token_program,
        // associated_token_program, // TODO: do we need it?
        sender,
        bump,
        config,
        user_counter,
        admin_rotation_state,
        revenue_mint,
        revenue_app_ata,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData {
        // bumps, // TODO: do we need it?
        rotation_timeout,
        account_registration_fee,
        account_data_size_range,
        ..
    } = InstructionData::try_from(instruction_data)?;

    let clock_time = get_clock_time()?;

    // devnet/mainnet program must be initialized by specified address
    if clock_time > CLOCK_TIME_MIN && sender.key() != &MAINNET_ADMIN {
        Err(AnyError::Auth(AuthError::Unauthorized))?;
    }

    // get bumps
    //
    let bump_seeds = &[SEED::BUMP];
    let (_bump_pda, bump_bump) = get_and_check_pda(bump_seeds, &crate::ID, Some(bump))?;

    let config_seeds = &[SEED::CONFIG];
    let (_config_pda, config_bump) = get_and_check_pda(config_seeds, &crate::ID, Some(config))?;

    let user_counter_seeds = &[SEED::USER_COUNTER];
    let (_user_counter_pda, user_counter_bump) =
        get_and_check_pda(user_counter_seeds, &crate::ID, Some(user_counter))?;

    let admin_rotation_state_seeds = &[SEED::ADMIN_ROTATION_STATE];
    let (_admin_rotation_state_pda, admin_rotation_state_bump) = get_and_check_pda(
        admin_rotation_state_seeds,
        &crate::ID,
        Some(admin_rotation_state),
    )?;

    // create and write pda
    //
    let bump_ref = &[bump_bump];
    let signer_seeds = &seeds!(SEED::BUMP, bump_ref);
    ProgramAccount::init(sender, bump, Bump::get_space(), signer_seeds, &crate::ID)?;
    AccountData::init(bump)?.save(Bump {
        config: config_bump,
        user_counter: user_counter_bump,
        rotation_state: admin_rotation_state_bump,
    })?;

    let bump_ref = &[config_bump];
    let signer_seeds = &seeds!(SEED::CONFIG, bump_ref);
    ProgramAccount::init(
        sender,
        config,
        Config::get_space(),
        signer_seeds,
        &crate::ID,
    )?;
    AccountData::init(config)?.save(Config {
        admin: *sender.key(),
        is_paused: false,
        rotation_timeout: rotation_timeout.unwrap_or(ROTATION_TIMEOUT),
        registration_fee: account_registration_fee.unwrap_or(AssetItem {
            amount: ACCOUNT_REGISTRATION_FEE_AMOUNT,
            asset: ACCOUNT_REGISTRATION_FEE_ASSET,
        }),
        data_size_range: account_data_size_range.unwrap_or(Range {
            min: ACCOUNT_DATA_SIZE_MIN,
            max: ACCOUNT_DATA_SIZE_MAX,
        }),
    })?;

    let bump_ref = &[user_counter_bump];
    let signer_seeds = &seeds!(SEED::USER_COUNTER, bump_ref);
    ProgramAccount::init(
        sender,
        user_counter,
        UserCounter::get_space(),
        signer_seeds,
        &crate::ID,
    )?;
    AccountData::init(user_counter)?.save(UserCounter::default())?;

    let bump_ref = &[admin_rotation_state_bump];
    let signer_seeds = &seeds!(SEED::ADMIN_ROTATION_STATE, bump_ref);
    ProgramAccount::init(
        sender,
        admin_rotation_state,
        RotationState::get_space(),
        signer_seeds,
        &crate::ID,
    )?;
    AccountData::init(bump)?.save(RotationState {
        owner: *sender.key(),
        new_owner: None,
        expiration_date: clock_time,
    })?;

    // create ata
    //
    AssociatedTokenAccount::init(
        sender,
        revenue_app_ata,
        revenue_mint,
        config,
        system_program,
        token_program,
    )?;

    Ok(())
}
