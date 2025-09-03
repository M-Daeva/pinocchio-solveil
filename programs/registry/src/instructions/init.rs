use {
    base::{
        accounts::{
            AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountInit, MintAccount,
            ProgramAccount, ProgramAccountInit, SignerAccount, SystemProgram,
        },
        converters::deserialize,
        error::AuthError,
        helpers::{get_and_check_pda, get_clock_time},
        types::Storage,
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
    let ix: &InstructionData = deserialize(instruction_data)?;
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

    SystemProgram::check(system_program)?;
    // token_program
    // associated_token_program
    SignerAccount::check(sender)?;
    // bump
    // config
    // user_counter
    // admin_rotation_state
    MintAccount::check(revenue_mint)?;
    // revenue_app_ata

    let clock_time = get_clock_time()?;

    // === use guards ===

    // devnet/mainnet program must be initialized by specified address
    if clock_time > CLOCK_TIME_MIN && sender.key() != &MAINNET_ADMIN {
        Err(AnyError::Auth(AuthError::Unauthorized))?;
    }

    // === init and write pda ===

    // config
    let (_, config_bump) = get_and_check_pda(&[SEED::CONFIG], &crate::ID, Some(config))?;
    ProgramAccount::init::<Config>(
        sender,
        config,
        &seeds!(SEED::CONFIG, &[config_bump]),
        &crate::ID,
    )?;
    Storage::init(config)?.update(|x| {
        let registration_fee = if ix.is_account_registration_fee() {
            ix.account_registration_fee
        } else {
            let mut x = AssetItem::default();
            x.set_amount(ACCOUNT_REGISTRATION_FEE_AMOUNT);
            x.asset = ACCOUNT_REGISTRATION_FEE_ASSET;
            x
        };

        let data_size_range = if ix.is_account_data_size_range() {
            ix.account_data_size_range
        } else {
            let mut x = Range::default();
            x.set_min(ACCOUNT_DATA_SIZE_MIN);
            x.set_max(ACCOUNT_DATA_SIZE_MAX);
            x
        };

        *x = Config::default();
        x.admin = *sender.key();
        x.set_rotation_timeout(ROTATION_TIMEOUT);
        x.registration_fee = registration_fee;
        x.data_size_range = data_size_range;
    })?;

    // user_counter
    let (_, user_counter_bump) =
        get_and_check_pda(&[SEED::USER_COUNTER], &crate::ID, Some(user_counter))?;
    ProgramAccount::init::<UserCounter>(
        sender,
        user_counter,
        &seeds!(SEED::USER_COUNTER, &[user_counter_bump]),
        &crate::ID,
    )?;
    Storage::init(user_counter)?.update(|x| {
        *x = UserCounter::default();
    })?;

    // admin_rotation_state
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
    Storage::<RotationState>::init(admin_rotation_state)?.update(|x| {
        x.owner = *sender.key();
        x.new_owner = *sender.key();
        x.set_expiration_date(clock_time);
    })?;

    // bump
    let (_, bump_bump) = get_and_check_pda(&[SEED::BUMP], &crate::ID, Some(bump))?;
    ProgramAccount::init::<Bump>(sender, bump, &seeds!(SEED::BUMP, &[bump_bump]), &crate::ID)?;
    Storage::<Bump>::init(bump)?.update(|x| {
        x.config = config_bump;
        x.user_counter = user_counter_bump;
        x.rotation_state = admin_rotation_state_bump;
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
