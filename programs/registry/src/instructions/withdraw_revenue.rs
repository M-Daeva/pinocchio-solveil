use {
    base::{
        error::AuthError,
        helpers::{get_ata_balance, get_token_decimals, transfer_token_from_program},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Bump, Config},
        types::withdraw_revenue::{Accounts, InstructionData},
    },
};

pub fn withdraw_revenue(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        sender,
        bump,
        config: config_acc,
        revenue_mint,
        revenue_recipient_ata,
        revenue_app_ata,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData { amount } = InstructionData::try_from(instruction_data)?;

    // get storages
    //
    let config = AccountData::<Config>::init(config_acc)?.load()?;

    // check sender
    if sender.key() != &config.admin {
        Err(AnyError::Auth(AuthError::Unauthorized))?;
    }

    // validate fee token
    if revenue_mint.key() != &config.registration_fee.asset {
        Err(AnyError::Custom(CustomError::WrongAssetType))?;
    }

    let app_balance = get_ata_balance(revenue_app_ata)?;
    let amount = amount.unwrap_or(app_balance);

    // lower limit of amount to withdraw
    if amount == 0 {
        Err(AnyError::Custom(CustomError::ZeroAmount))?;
    }

    // higher limit of amount to withdraw
    if amount > app_balance {
        Err(AnyError::Custom(CustomError::ExceededAvailableAssetAmount))?;
    }

    let bump = AccountData::<Bump>::init(bump)?.load()?;
    let bump_ref = &[bump.config];
    let signer_seeds = &seeds!(SEED::CONFIG, bump_ref);
    transfer_token_from_program(
        amount,
        revenue_mint,
        revenue_app_ata,
        revenue_recipient_ata,
        signer_seeds,
        config_acc,
        get_token_decimals(revenue_mint)?,
    )?;

    // TODO: init_if_needed for revenue_recipient_ata?

    Ok(())
}
