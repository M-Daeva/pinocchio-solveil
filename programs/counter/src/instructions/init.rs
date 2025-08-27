use {
    base::{
        accounts::{ProgramAccount, ProgramAccountInit},
        helpers::get_and_check_pda,
        types::{AccountData, Space},
    },
    counter_cpi::{
        state::{seed as SEED, Counter},
        types::init::{Accounts, InstructionData},
    },
    pinocchio::{account_info::AccountInfo, seeds, ProgramResult},
};

pub fn init(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        sender,
        counter_account,
        ..
    } = Accounts::try_from(accounts)?;

    let value = InstructionData::try_from(instruction_data)?
        .value
        .unwrap_or_default();

    let seeds = &[SEED::COUNTER];
    let (_pda, bump) = get_and_check_pda(seeds, &crate::ID, Some(counter_account))?;

    let bump_ref = &[bump];
    let signer_seeds = &seeds!(SEED::COUNTER, bump_ref);
    ProgramAccount::init(
        sender,
        counter_account,
        Counter::get_space(),
        signer_seeds,
        &crate::ID,
    )?;

    AccountData::init(counter_account)?.save(Counter { value })?;

    Ok(())
}
