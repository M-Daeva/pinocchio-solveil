use {
    base::{
        accounts::{AccountCheck, SignerAccount, SystemProgram},
        converters::{to_option, to_u8},
        guards::check_ix_data_len,
        types::Result,
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
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [sender, counter_account, system_program] = accounts else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SignerAccount::check(sender)?;
        SystemProgram::check(system_program)?;

        Ok(Self {
            sender,
            counter_account,
            system_program,
        })
    }
}

pub struct InstructionData {
    pub value: Option<u8>,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        let (value, idx) = to_option(data, 0, to_u8)?;
        check_ix_data_len(data, idx)?;

        Ok(Self { value })
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok([
            vec![crate::state::discriminator::INIT],
            base::converters::from_u8_option(self.value)?,
        ]
        .concat())
    }
}
