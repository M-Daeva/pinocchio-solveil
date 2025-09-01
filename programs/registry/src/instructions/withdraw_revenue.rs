use {
    base::{
        accounts::{
            AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, MintAccount,
            ProgramAccount, ProgramAccountCheck, SignerAccount, SystemProgram,
        },
        error::AuthError,
        helpers::{get_ata_balance, get_token_decimals, transfer_token_from_program},
        types::{AccountData, ZeroCopyDeserialize},
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Bump, Config},
        types::withdraw_revenue::{Accounts, InstructionData},
    },
};

pub fn withdraw_revenue(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let InstructionData { amount } = InstructionData::deserialize_from(instruction_data, 0)?.0;

    let Accounts {
        system_program,
        token_program,
        sender,
        recipient,
        bump,
        config,
        revenue_mint,
        revenue_recipient_ata,
        revenue_app_ata,
        ..
    } = Accounts::try_from(accounts)?;

    SystemProgram::check(system_program)?;
    // token_program,
    // associated_token_program,
    SignerAccount::check(sender)?;
    // recipient,
    ProgramAccount::check::<Bump>(bump, &crate::ID, Some(&[SEED::BUMP]))?;
    ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
    MintAccount::check(revenue_mint)?;
    AssociatedTokenAccount::check(
        revenue_recipient_ata,
        recipient,
        revenue_mint,
        token_program,
    )?;
    AssociatedTokenAccount::check(revenue_app_ata, config, revenue_mint, token_program)?;

    let config_acc = config;
    let app_balance = get_ata_balance(revenue_app_ata)?;
    let amount = amount.unwrap_or(app_balance);

    // === load storages ===

    let bump = AccountData::<Bump>::init(bump)?.load()?;
    let config = AccountData::<Config>::init(config_acc)?.load()?;

    // === use guards ===

    // check sender
    if sender.key() != &config.admin {
        Err(AnyError::Auth(AuthError::Unauthorized))?;
    }

    // validate fee token
    if revenue_mint.key() != &config.registration_fee.asset {
        Err(AnyError::Custom(CustomError::WrongAssetType))?;
    }

    // lower limit of amount to withdraw
    if amount == 0 {
        Err(AnyError::Custom(CustomError::ZeroAmount))?;
    }

    // higher limit of amount to withdraw
    if amount > app_balance {
        Err(AnyError::Custom(CustomError::ExceededAvailableAssetAmount))?;
    }

    // === transfer tokens from app to sender ===

    transfer_token_from_program(
        amount,
        revenue_mint,
        revenue_app_ata,
        revenue_recipient_ata,
        &seeds!(SEED::CONFIG, &[bump.config]),
        config_acc,
        get_token_decimals(revenue_mint)?,
    )?;

    Ok(())
}
