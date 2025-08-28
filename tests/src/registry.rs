use {
    crate::helpers::{
        extensions::registry::CounterExtension,
        suite::{
            core::App,
            types::{AppToken, AppUser, PinPubkey, TestResult},
        },
    },
    pretty_assertions::assert_eq,
    registry_cpi::{
        state::{
            Config, RotationState, UserCounter, ACCOUNT_DATA_SIZE_MAX, ACCOUNT_DATA_SIZE_MIN,
            ACCOUNT_REGISTRATION_FEE_AMOUNT, ROTATION_TIMEOUT,
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
