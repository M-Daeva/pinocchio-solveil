use {
    base::{
        accounts::{
            AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, MintAccount,
            ProgramAccount, ProgramAccountCheck, SignerAccount, SystemProgram,
        },
        helpers::{get_token_decimals, transfer_token_from_user},
        types::{StorageR, StorageW},
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
    registry_cpi::{
        error::{AnyError, CustomError},
        state::{seed as SEED, Bump, Config, UserId},
        types::activate_account::Accounts,
    },
};

pub fn activate_account(accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    // let ix: &InstructionData = deserialize(instruction_data)?;
    let Accounts {
        system_program,
        token_program,
        sender,
        bump,
        config,
        user_id,
        revenue_mint,
        revenue_sender_ata,
        revenue_app_ata,
        ..
    } = Accounts::try_from(accounts)?;

    SystemProgram::check(system_program)?;
    // token_program,
    // associated_token_program,
    SignerAccount::check(sender)?;
    ProgramAccount::check::<Bump>(bump, &crate::ID, Some(&[SEED::BUMP]))?;
    ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
    // ProgramAccount::check( // TODO: check with user instead of sender
    //     user_id,
    //     &crate::ID,
    //     UserId::get_space(),
    //     Some(&[SEED::USER_ID, sender.key()]),
    // )?;
    MintAccount::check(revenue_mint)?;
    AssociatedTokenAccount::check(revenue_sender_ata, sender, revenue_mint, token_program)?;
    AssociatedTokenAccount::check(revenue_app_ata, config, revenue_mint, token_program)?;

    // === load storages ===

    let config = StorageR::<Config>::load(config)?;

    StorageW::<UserId>::init(user_id)?.update(|user_id| {
        // === use guards ===

        // only open account can be activated
        if !user_id.get_is_open_flag() {
            // TODO: Err(CustomError::AccountIsNotOpened)?;
            Err(AnyError::Custom(CustomError::AccountIsNotOpened))?;
        }

        // only inactive account can be activated
        if user_id.get_is_activated_flag() {
            Err(AnyError::Custom(CustomError::ActivateAccountTwice))?;
        }

        // validate fee token
        if revenue_mint.key() != &config.registration_fee.asset {
            Err(AnyError::Custom(CustomError::WrongAssetType))?;
        }

        // === save storages ===

        user_id.set_is_activated_flag(true);
        Ok(())
    })?;

    // === transfer tokens from user to app ===

    transfer_token_from_user(
        config.registration_fee.amount.get(),
        revenue_mint,
        revenue_sender_ata,
        revenue_app_ata,
        sender,
        get_token_decimals(revenue_mint)?,
    )
}
