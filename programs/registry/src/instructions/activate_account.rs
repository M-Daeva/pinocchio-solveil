use {
    base::{
        helpers::{get_token_decimals, transfer_token_from_user},
        types::AccountData,
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{Config, UserId},
        types::activate_account::{Accounts, InstructionData},
    },
};

pub fn activate_account(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        sender,
        config,
        user_id,
        revenue_mint,
        revenue_sender_ata,
        revenue_app_ata,
        ..
    } = Accounts::try_from(accounts)?;

    let InstructionData { .. } = InstructionData::try_from(instruction_data)?;

    // get storages
    //
    let config = AccountData::<Config>::init(config)?.load()?;

    let mut user_id_storage = AccountData::<UserId>::init(user_id)?;
    let mut user_id = user_id_storage.load()?;

    // only open account can be activated
    if !user_id.is_open {
        Err(AnyError::Custom(CustomError::AccountIsNotOpened))?;
    }

    // only inactive account can be activated
    if user_id.is_activated {
        Err(AnyError::Custom(CustomError::ActivateAccountTwice))?;
    }

    // validate fee token
    if revenue_mint.key() != &config.registration_fee.asset {
        Err(AnyError::Custom(CustomError::WrongAssetType))?;
    }

    user_id.is_activated = true;
    user_id_storage.save(user_id)?;

    transfer_token_from_user(
        config.registration_fee.amount,
        revenue_mint,
        revenue_sender_ata,
        revenue_app_ata,
        sender,
        get_token_decimals(revenue_mint)?,
    )?;

    Ok(())
}
