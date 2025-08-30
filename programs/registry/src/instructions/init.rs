use {
    base::{
        accounts::{
            AssociatedTokenAccount, AssociatedTokenAccountInit, ProgramAccount, ProgramAccountInit,
        },
        error::AuthError,
        helpers::{get_and_check_pda, get_clock_time},
        types::AccountData,
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

    // === init and write pda ===

    // config pda
    let (_, config_bump) = get_and_check_pda(&[SEED::CONFIG], &crate::ID, Some(config))?;
    ProgramAccount::init::<Config>(
        sender,
        config,
        &seeds!(SEED::CONFIG, &[config_bump]),
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

    // user_counter pda
    let (_, user_counter_bump) =
        get_and_check_pda(&[SEED::USER_COUNTER], &crate::ID, Some(user_counter))?;
    ProgramAccount::init::<UserCounter>(
        sender,
        user_counter,
        &seeds!(SEED::USER_COUNTER, &[user_counter_bump]),
        &crate::ID,
    )?;
    AccountData::init(user_counter)?.save(UserCounter::default())?;

    // admin_rotation_state pda
    let (_, admin_rotation_state_bump) = get_and_check_pda(
        &[SEED::ADMIN_ROTATION_STATE],
        &crate::ID,
        Some(admin_rotation_state),
    )?;
    ProgramAccount::init::<RotationState>(
        sender,
        admin_rotation_state,
        &seeds!(SEED::ADMIN_ROTATION_STATE, &[admin_rotation_state_bump]),
        &crate::ID,
    )?;
    AccountData::init(admin_rotation_state)?.save(RotationState {
        owner: *sender.key(),
        new_owner: None,
        expiration_date: clock_time,
    })?;

    // bump pda
    let (_, bump_bump) = get_and_check_pda(&[SEED::BUMP], &crate::ID, Some(bump))?;
    ProgramAccount::init::<Bump>(sender, bump, &seeds!(SEED::BUMP, &[bump_bump]), &crate::ID)?;
    AccountData::init(bump)?.save(Bump {
        config: config_bump,
        user_counter: user_counter_bump,
        rotation_state: admin_rotation_state_bump,
    })?;

    // === init ata ===

    // revenue_app_ata
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
