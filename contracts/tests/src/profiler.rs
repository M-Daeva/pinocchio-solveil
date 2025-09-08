use {
    crate::helpers::{
        extensions::registry::RegistryExtension,
        suite::{
            core::{get_program_size, App, PROGRAM_NAME_REGISTRY},
            types::{AppToken, AppUser, PinPubkey, TestResult},
        },
    },
    base::types::Uint64,
    registry_cpi::{
        state::{ACCOUNT_DATA_SIZE_MAX, ACCOUNT_REGISTRATION_FEE_AMOUNT},
        types::common::AssetItem,
    },
};

#[test]
fn profiling_registry() -> TestResult<()> {
    const DATA_0: &str = "encrypted_secrets_0";
    const NONCE_0: u64 = 1;

    let mut app = App::new(false);

    let init_cu = app
        .registry_try_init(
            AppUser::Admin,
            None,
            Some(AssetItem {
                amount: Uint64::from(ACCOUNT_REGISTRATION_FEE_AMOUNT),
                asset: AppToken::USDC.pubkey(),
            }),
            None,
        )?
        .compute_units_consumed;

    let create_account_cu = app
        .registry_try_create_account(AppUser::Alice, ACCOUNT_DATA_SIZE_MAX, None)?
        .compute_units_consumed;
    let activate_account_cu = app
        .registry_try_activate_account(AppUser::Alice, None, None)?
        .compute_units_consumed;
    let write_data_cu = app
        .registry_try_write_data(AppUser::Alice, DATA_0, NONCE_0)?
        .compute_units_consumed;

    const PROGRAM_NAME: &str = PROGRAM_NAME_REGISTRY;
    const PREVIOUS_RESULT: &str = "
==================== previous result ====================
registry program size: 49240 B
registry init instruction: 36700 CU
registry create_account instruction: 19114 CU
registry activate_account instruction: 14222 CU
registry write_data instruction: 4785 CU
==================== current result =====================";

    println!("{}", PREVIOUS_RESULT);
    println!(
        "{} program size: {} B",
        PROGRAM_NAME,
        get_program_size(PROGRAM_NAME)?
    );
    println!("{} init instruction: {} CU", PROGRAM_NAME, init_cu);
    println!(
        "{} create_account instruction: {} CU",
        PROGRAM_NAME, create_account_cu
    );
    println!(
        "{} activate_account instruction: {} CU",
        PROGRAM_NAME, activate_account_cu
    );
    println!(
        "{} write_data instruction: {} CU",
        PROGRAM_NAME, write_data_cu
    );

    Ok(())
}
