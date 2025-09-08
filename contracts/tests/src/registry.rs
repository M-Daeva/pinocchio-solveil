use {
    crate::helpers::{
        extensions::registry::{get_data_buffer, RegistryExtension},
        suite::{
            core::{assert_error, App},
            types::{pin_to_sol_pubkey, AppToken, AppUser, PinPubkey, TestResult},
        },
    },
    base::{
        error::AuthError,
        types::{BitField, Uint32, Uint64},
    },
    pretty_assertions::assert_eq,
    registry_cpi::{
        error::CustomError,
        state::{
            Config, RotationState, UserAccount, UserCounter, ACCOUNT_DATA_SIZE_MAX,
            ACCOUNT_DATA_SIZE_MIN, ACCOUNT_REGISTRATION_FEE_AMOUNT, CLOCK_TIME_MIN,
            ROTATION_TIMEOUT,
        },
        types::common::{AssetItem, Range},
    },
};

fn init_app(is_log_displayed: bool) -> TestResult<App> {
    let mut app = App::new(is_log_displayed);

    app.registry_try_init(
        AppUser::Admin,
        None,
        Some(AssetItem {
            amount: Uint64::from(ACCOUNT_REGISTRATION_FEE_AMOUNT),
            asset: AppToken::USDC.pubkey(),
        }),
        None,
    )?;

    Ok(app)
}

#[test]
fn init_default() -> TestResult<()> {
    let app = init_app(false)?;

    assert_eq!(
        app.registry_query_config()?,
        Config {
            admin: AppUser::Admin.pubkey(),
            is_paused: BitField::default(),
            rotation_timeout: Uint32::from(ROTATION_TIMEOUT),
            registration_fee: AssetItem {
                amount: Uint64::from(ACCOUNT_REGISTRATION_FEE_AMOUNT),
                asset: AppToken::USDC.pubkey(),
            },
            data_size_range: Range {
                min: Uint32::from(ACCOUNT_DATA_SIZE_MIN),
                max: Uint32::from(ACCOUNT_DATA_SIZE_MAX),
            }
        }
    );
    assert_eq!(app.registry_query_user_counter()?, UserCounter::default());
    assert_eq!(
        app.registry_query_admin_rotation_state()?,
        RotationState {
            owner: AppUser::Admin.pubkey(),
            new_owner: AppUser::Admin.pubkey(),
            expiration_date: Uint64::from(app.get_clock_time())
        }
    );

    Ok(())
}

#[test]
fn init_admin_guard() -> TestResult<()> {
    let mut app = App::new(false);
    app.wait(CLOCK_TIME_MIN + 1);

    // only specified admin can init devnet/mainnet program
    let res = app
        .registry_try_init(
            AppUser::Admin,
            None,
            Some(AssetItem {
                amount: Uint64::from(ACCOUNT_REGISTRATION_FEE_AMOUNT),
                asset: AppToken::USDC.pubkey(),
            }),
            None,
        )
        .unwrap_err();
    assert_error(res, AuthError::Unauthorized);

    Ok(())
}

#[test]
fn update_config_default() -> TestResult<()> {
    let mut app = init_app(false)?;

    app.registry_try_update_config(
        AppUser::Admin,
        None,
        None,
        None,
        None,
        Some(Range {
            min: Uint32::from(2 * ACCOUNT_DATA_SIZE_MIN),
            max: Uint32::from(ACCOUNT_DATA_SIZE_MAX),
        }),
    )?;

    assert_eq!(
        app.registry_query_config()?,
        Config {
            admin: AppUser::Admin.pubkey(),
            is_paused: BitField::from(false),
            rotation_timeout: Uint32::from(ROTATION_TIMEOUT),
            registration_fee: AssetItem {
                amount: Uint64::from(ACCOUNT_REGISTRATION_FEE_AMOUNT),
                asset: AppToken::USDC.pubkey(),
            },
            data_size_range: Range {
                min: Uint32::from(2 * ACCOUNT_DATA_SIZE_MIN),
                max: Uint32::from(ACCOUNT_DATA_SIZE_MAX),
            }
        }
    );

    Ok(())
}

#[test]
fn transfer_admin() -> TestResult<()> {
    let mut app = init_app(false)?;

    // only admin can rotate admin
    let res = app
        .registry_try_update_config(AppUser::Alice, Some(AppUser::Alice), None, None, None, None)
        .unwrap_err();
    assert_error(res, AuthError::Unauthorized);

    // new admin isn't specified
    let res = app
        .registry_try_confirm_admin_rotation(AppUser::Alice)
        .unwrap_err();
    assert_error(res, AuthError::NoNewOwner);

    // the admin can't be new admin
    let res = app
        .registry_try_update_config(AppUser::Admin, Some(AppUser::Admin), None, None, None, None)
        .unwrap_err();
    assert_error(res, AuthError::UselessRotation);

    // too late to confirm admin rotation
    app.registry_try_update_config(AppUser::Admin, Some(AppUser::Alice), None, None, None, None)?;
    app.wait(ROTATION_TIMEOUT as u64);
    let res = app
        .registry_try_confirm_admin_rotation(AppUser::Alice)
        .unwrap_err();
    assert_error(res, AuthError::TransferOwnerDeadline);

    // only new admin can confirm admin rotation
    app.registry_try_update_config(AppUser::Admin, Some(AppUser::Alice), None, None, None, None)?;
    let res = app
        .registry_try_confirm_admin_rotation(AppUser::Bob)
        .unwrap_err();
    assert_error(res, AuthError::Unauthorized);

    // success
    app.registry_try_confirm_admin_rotation(AppUser::Alice)?;
    assert_eq!(app.registry_query_config()?.admin, AppUser::Alice.pubkey());

    // new admin isn't specified after rotation
    let res = app
        .registry_try_confirm_admin_rotation(AppUser::Admin)
        .unwrap_err();
    assert_error(res, AuthError::NoNewOwner);

    Ok(())
}

#[test]
fn create_account_guards() -> TestResult<()> {
    let mut app = init_app(false)?;

    // user can't create account with improper user_id
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, Some(5))
        .unwrap_err();

    // user can't create account with too small data
    let res = app
        .registry_try_create_account(AppUser::Alice, 1, None)
        .unwrap_err();
    assert_error(res, CustomError::MaxDataSizeIsOutOfRange);

    // user can't create account with too much data
    let res = app
        .registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX + 1, None)
        .unwrap_err();
    assert_error(res, CustomError::MaxDataSizeIsOutOfRange);

    // user can't create account when program is paused
    app.registry_try_update_config(AppUser::Admin, None, Some(true), None, None, None)?;
    let res = app
        .registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)
        .unwrap_err();
    app.registry_try_update_config(AppUser::Admin, None, Some(false), None, None, None)?;
    assert_error(res, CustomError::ContractIsPaused);

    // user can't create account twice
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    // 1) with the same user_id
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, Some(1))
        .unwrap_err();
    // 2) with a new user_id
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)
        .unwrap_err();
    // 3) even if it's closed
    app.registry_try_close_account(AppUser::Alice, None)?;
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, Some(1))
        .unwrap_err();
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)
        .unwrap_err();
    app.registry_try_reopen_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX)?;

    // other user can't create account with the same user_id
    app.registry_try_create_account(AppUser::Bob, ACCOUNT_DATA_SIZE_MAX, Some(1))
        .unwrap_err();

    Ok(())
}

#[test]
fn create_and_activate_account_default() -> TestResult<()> {
    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;

    let user_id = app.registry_query_user_id(AppUser::Alice)?;
    assert_eq!(user_id.id.get(), 1);
    assert_eq!(user_id.get_is_open_flag(), true);
    assert_eq!(user_id.get_is_activated_flag(), false);

    assert_eq!(
        app.registry_query_user_account(AppUser::Alice)?,
        UserAccount {
            data: UserAccount::default().data,
            nonce: Uint64::default(),
            max_size: ACCOUNT_DATA_SIZE_MAX.into()
        }
    );

    let alice_usdc_before = app.get_balance(AppUser::Alice, AppToken::USDC);
    app.registry_try_activate_account(AppUser::Alice, None, None)?;

    let alice_usdc_after = app.get_balance(AppUser::Alice, AppToken::USDC);
    assert_eq!(
        alice_usdc_before - alice_usdc_after,
        ACCOUNT_REGISTRATION_FEE_AMOUNT
    );

    let user_id = app.registry_query_user_id(AppUser::Alice)?;
    assert_eq!(user_id.id.get(), 1);
    assert_eq!(user_id.get_is_open_flag(), true);
    assert_eq!(user_id.get_is_activated_flag(), true);

    Ok(())
}

#[test]
fn activate_account_guards() -> TestResult<()> {
    let mut app = init_app(false)?;

    // user can't activate nonexistent account
    app.registry_try_activate_account(AppUser::Alice, None, None)
        .unwrap_err();

    // user can't activate closed account
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_close_account(AppUser::Alice, None)?;
    let res = app
        .registry_try_activate_account(AppUser::Alice, None, None)
        .unwrap_err();
    assert_error(res, CustomError::AccountIsNotOpened);

    // user can't activate account using wrong token
    app.registry_try_reopen_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX)?;
    app.get_or_create_ata(
        AppUser::Admin,
        &app.pda.registry_config(),
        &pin_to_sol_pubkey(&AppToken::PYTH.pubkey()),
    )?;
    let res = app
        .registry_try_activate_account(AppUser::Alice, None, Some(AppToken::PYTH))
        .unwrap_err();
    assert_error(res, CustomError::WrongAssetType);

    // user can't activate account twice
    app.registry_try_activate_account(AppUser::Alice, None, None)?;
    let res = app
        .registry_try_activate_account(AppUser::Alice, None, None)
        .unwrap_err();
    assert_error(res, CustomError::ActivateAccountTwice);

    Ok(())
}

#[test]
fn activate_account_for_other_user() -> TestResult<()> {
    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_activate_account(AppUser::Bob, Some(AppUser::Alice), None)?;

    assert_eq!(
        app.registry_query_user_account(AppUser::Alice)?.max_size,
        ACCOUNT_DATA_SIZE_MAX.into()
    );

    Ok(())
}

#[test]
fn withdraw_revenue_default() -> TestResult<()> {
    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_activate_account(AppUser::Alice, None, None)?;

    let admin_usdc_before = app.get_balance(AppUser::Admin, AppToken::USDC);

    app.registry_try_withdraw_revenue(AppUser::Admin, None, None, None)?;

    let admin_usdc_after = app.get_balance(AppUser::Admin, AppToken::USDC);
    assert_eq!(
        admin_usdc_after - admin_usdc_before,
        ACCOUNT_REGISTRATION_FEE_AMOUNT
    );

    Ok(())
}

#[test]
fn withdraw_revenue_by_amount_to_recipient() -> TestResult<()> {
    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_activate_account(AppUser::Alice, None, None)?;

    let revenue = app.get_ata_token_balance(
        &app.pda.registry_config(),
        &pin_to_sol_pubkey(&AppToken::USDC.pubkey()),
    );
    assert_eq!(revenue, ACCOUNT_REGISTRATION_FEE_AMOUNT);

    let bob_usdc_before = app.get_balance(AppUser::Bob, AppToken::USDC);
    app.registry_try_withdraw_revenue(
        AppUser::Admin,
        Some(revenue / 2),
        Some(AppUser::Bob),
        Some(AppToken::USDC),
    )?;

    let bob_usdc_after = app.get_balance(AppUser::Bob, AppToken::USDC);
    assert_eq!(
        bob_usdc_after - bob_usdc_before,
        ACCOUNT_REGISTRATION_FEE_AMOUNT / 2
    );

    Ok(())
}

#[test]
fn withdraw_revenue_guards() -> TestResult<()> {
    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_activate_account(AppUser::Alice, None, None)?;

    // only admin can withdraw
    let res = app
        .registry_try_withdraw_revenue(AppUser::Alice, None, None, None)
        .unwrap_err();
    assert_error(res, AuthError::Unauthorized);

    // lower amount limit
    let res = app
        .registry_try_withdraw_revenue(AppUser::Admin, Some(0), None, None)
        .unwrap_err();
    assert_error(res, CustomError::ZeroAmount);

    // higher amount limit
    let res = app
        .registry_try_withdraw_revenue(
            AppUser::Admin,
            Some(ACCOUNT_REGISTRATION_FEE_AMOUNT + 1),
            None,
            None,
        )
        .unwrap_err();
    assert_error(res, CustomError::ExceededAvailableAssetAmount);

    app.get_or_create_ata(
        AppUser::Admin,
        &app.pda.registry_config(),
        &pin_to_sol_pubkey(&AppToken::PYTH.pubkey()),
    )?;
    let res = app
        .registry_try_withdraw_revenue(AppUser::Admin, None, None, Some(AppToken::PYTH))
        .unwrap_err();
    assert_error(res, CustomError::WrongAssetType);

    Ok(())
}

#[test]
fn close_account_guards() -> TestResult<()> {
    let mut app = init_app(false)?;

    // account must be created first
    app.registry_try_close_account(AppUser::Alice, None)
        .unwrap_err();

    // close account of other user
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_close_account(AppUser::Bob, Some(AppUser::Alice))
        .unwrap_err();

    // close account twice
    app.registry_try_close_account(AppUser::Alice, None)?;
    app.registry_try_close_account(AppUser::Alice, None)
        .unwrap_err();

    Ok(())
}

#[test]
fn reopen_account_default() -> TestResult<()> {
    const ACCOUNT_DATA_SIZE_MAX_0: u32 = 1_000;
    const ACCOUNT_DATA_SIZE_MAX_1: u32 = 1_000;

    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX_0, None)?;
    app.registry_try_activate_account(AppUser::Alice, None, None)?;
    app.registry_try_close_account(AppUser::Alice, None)?;

    let user_id = app.registry_query_user_id(AppUser::Alice)?;
    assert_eq!(user_id.id.get(), 1);
    assert_eq!(user_id.get_is_open_flag(), false);
    assert_eq!(user_id.get_is_activated_flag(), true);

    app.registry_try_reopen_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX_1)?;

    let user_id = app.registry_query_user_id(AppUser::Alice)?;
    assert_eq!(user_id.id.get(), 1);
    assert_eq!(user_id.get_is_open_flag(), true);
    assert_eq!(user_id.get_is_activated_flag(), true);

    assert_eq!(
        app.registry_query_user_account(AppUser::Alice)?.max_size,
        ACCOUNT_DATA_SIZE_MAX_1.into()
    );

    Ok(())
}

#[test]
fn reopen_account_guards() -> TestResult<()> {
    let mut app = init_app(false)?;

    // account must be created first
    app.registry_try_reopen_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX)
        .unwrap_err();

    // account can't be open twice
    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_reopen_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX)
        .unwrap_err();

    Ok(())
}

#[test]
fn write_data_default() -> TestResult<()> {
    const DATA_0: &str = "encrypted_secrets_0";
    const DATA_1: &str = "encrypted_secrets_1";
    const NONCE_0: u64 = 1;
    const NONCE_1: u64 = 2;

    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_activate_account(AppUser::Alice, None, None)?;

    for (data, nonce) in [(DATA_0, NONCE_0), (DATA_1, NONCE_1)] {
        app.registry_try_write_data(AppUser::Alice, data, nonce)?;

        assert_eq!(
            app.registry_query_user_account(AppUser::Alice)?,
            UserAccount {
                data: get_data_buffer(data),
                nonce: Uint64::from(nonce),
                max_size: ACCOUNT_DATA_SIZE_MAX.into()
            }
        );
    }

    Ok(())
}

#[test]
fn write_data_multiple_users() -> TestResult<()> {
    const DATA_0: &str = "encrypted_secrets_0";
    const DATA_1: &str = "encrypted_secrets_1";
    const NONCE_0: u64 = 1;
    const NONCE_1: u64 = 2;

    let mut app = init_app(false)?;

    for user in [AppUser::Alice, AppUser::Bob] {
        app.registry_try_create_account(user, ACCOUNT_DATA_SIZE_MAX, None)?;
        app.registry_try_activate_account(user, None, None)?;
    }

    app.registry_try_write_data(AppUser::Alice, DATA_0, NONCE_0)?;
    app.registry_try_write_data(AppUser::Bob, DATA_1, NONCE_1)?;

    assert_eq!(
        app.registry_query_user_account(AppUser::Alice)?,
        UserAccount {
            data: get_data_buffer(DATA_0),
            nonce: NONCE_0.into(),
            max_size: ACCOUNT_DATA_SIZE_MAX.into()
        }
    );
    assert_eq!(
        app.registry_query_user_account(AppUser::Bob)?,
        UserAccount {
            data: get_data_buffer(DATA_1),
            nonce: NONCE_1.into(),
            max_size: ACCOUNT_DATA_SIZE_MAX.into()
        }
    );

    Ok(())
}

#[test]
fn rotate_account() -> TestResult<()> {
    const DATA_0: &str = "encrypted_secrets_0";
    const NONCE_0: u64 = 1;

    let mut app = init_app(false)?;

    app.registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?;
    app.registry_try_activate_account(AppUser::Alice, None, None)?;
    app.registry_try_write_data(AppUser::Alice, DATA_0, NONCE_0)?;

    // new owner isn't specified
    let res = app
        .registry_try_confirm_account_rotation(AppUser::Bob, AppUser::Alice)
        .unwrap_err();
    assert_error(res, AuthError::NoNewOwner);

    // the owner can't be new owner
    let res = app
        .registry_try_request_account_rotation(AppUser::Alice, AppUser::Alice)
        .unwrap_err();
    assert_error(res, AuthError::UselessRotation);

    // too late to confirm account rotation
    app.registry_try_request_account_rotation(AppUser::Alice, AppUser::Bob)?;
    app.wait(ROTATION_TIMEOUT as u64);
    let res = app
        .registry_try_confirm_account_rotation(AppUser::Bob, AppUser::Alice)
        .unwrap_err();
    assert_error(res, AuthError::TransferOwnerDeadline);

    // only new owner can confirm account rotation
    app.registry_try_request_account_rotation(AppUser::Alice, AppUser::Bob)?;
    let res = app
        .registry_try_confirm_account_rotation(AppUser::Admin, AppUser::Alice)
        .unwrap_err();
    assert_error(res, AuthError::Unauthorized);

    // success
    app.registry_try_confirm_account_rotation(AppUser::Bob, AppUser::Alice)?;
    app.registry_query_user_id(AppUser::Alice).unwrap_err();
    assert_eq!(
        app.registry_query_user_account(AppUser::Bob)?,
        UserAccount {
            data: get_data_buffer(DATA_0),
            nonce: NONCE_0.into(),
            max_size: ACCOUNT_DATA_SIZE_MAX.into()
        }
    );

    // new owner isn't specified after rotation
    let res = app
        .registry_try_confirm_account_rotation(AppUser::Alice, AppUser::Bob)
        .unwrap_err();
    assert_error(res, AuthError::NoNewOwner);

    Ok(())
}
