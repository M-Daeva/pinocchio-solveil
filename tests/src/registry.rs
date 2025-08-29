use {
    crate::helpers::{
        extensions::registry::CounterExtension,
        suite::{
            core::{assert_error, App},
            types::{AppToken, AppUser, PinPubkey, TestResult},
        },
    },
    base::error::AuthError,
    pretty_assertions::assert_eq,
    registry_cpi::{
        state::{
            Config, RotationState, UserCounter, ACCOUNT_DATA_SIZE_MAX, ACCOUNT_DATA_SIZE_MIN,
            ACCOUNT_REGISTRATION_FEE_AMOUNT, CLOCK_TIME_MIN, ROTATION_TIMEOUT,
        },
        types::common::{AssetItem, Range},
    },
};

fn init_app() -> TestResult<App> {
    let mut app = App::new(false);

    app.registry_try_init(
        AppUser::Admin,
        None,
        Some(AssetItem {
            amount: ACCOUNT_REGISTRATION_FEE_AMOUNT,
            asset: AppToken::USDC.pubkey(),
        }),
        None,
    )?;

    Ok(app)
}

#[test]
fn init_default() -> TestResult<()> {
    let app = init_app()?;

    assert_eq!(
        app.registry_query_config()?,
        Config {
            admin: AppUser::Admin.pubkey(),
            is_paused: false,
            rotation_timeout: ROTATION_TIMEOUT,
            registration_fee: AssetItem {
                amount: ACCOUNT_REGISTRATION_FEE_AMOUNT,
                asset: AppToken::USDC.pubkey(),
            },
            data_size_range: Range {
                min: ACCOUNT_DATA_SIZE_MIN,
                max: ACCOUNT_DATA_SIZE_MAX,
            }
        }
    );
    assert_eq!(app.registry_query_user_counter()?, UserCounter::default());
    assert_eq!(
        app.registry_query_admin_rotation_state()?,
        RotationState {
            owner: AppUser::Admin.pubkey(),
            new_owner: None,
            expiration_date: app.get_clock_time()
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
                amount: ACCOUNT_REGISTRATION_FEE_AMOUNT,
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
    let mut app = init_app()?;

    app.registry_try_update_config(
        AppUser::Admin,
        None,
        None,
        None,
        None,
        Some(Range {
            min: 2 * ACCOUNT_DATA_SIZE_MIN,
            max: ACCOUNT_DATA_SIZE_MAX,
        }),
    )?;

    assert_eq!(
        app.registry_query_config()?,
        Config {
            admin: AppUser::Admin.pubkey(),
            is_paused: false,
            rotation_timeout: ROTATION_TIMEOUT,
            registration_fee: AssetItem {
                amount: ACCOUNT_REGISTRATION_FEE_AMOUNT,
                asset: AppToken::USDC.pubkey(),
            },
            data_size_range: Range {
                min: 2 * ACCOUNT_DATA_SIZE_MIN,
                max: ACCOUNT_DATA_SIZE_MAX,
            }
        }
    );

    Ok(())
}

#[test]
fn transfer_admin() -> TestResult<()> {
    let mut app = init_app()?;

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
