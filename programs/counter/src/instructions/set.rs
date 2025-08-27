use {
    base::types::AccountData,
    counter_cpi::{
        state::Counter,
        types::set::{Accounts, InstructionData},
    },
    pinocchio::{account_info::AccountInfo, ProgramResult},
};

pub fn set(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let Accounts {
        counter_account, ..
    } = Accounts::try_from(accounts)?;

    let InstructionData { value } = InstructionData::try_from(instruction_data)?;

    AccountData::init(counter_account)?.save(Counter { value })?;

    Ok(())
}
