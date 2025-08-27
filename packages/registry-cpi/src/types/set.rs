use {
    crate::state::{seed as SEED, Counter},
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        converters::to_u8,
        guards::check_ix_data_len,
        types::{Result, Space},
    },
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountMetas)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,
    #[account(writable)]
    pub counter_account: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [sender, counter_account] = accounts else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SignerAccount::check(sender)?;
        ProgramAccount::check(
            counter_account,
            &crate::ID,
            Counter::get_space(),
            Some(&[SEED::COUNTER]),
        )?;

        Ok(Self {
            sender,
            counter_account,
        })
    }
}

pub struct InstructionData {
    pub value: u8,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        let (value, idx) = to_u8(data, 0)?;
        check_ix_data_len(data, idx)?;

        Ok(Self { value })
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok([
            vec![crate::state::discriminator::SET],
            base::converters::from_u8(self.value)?,
        ]
        .concat())
    }
}
