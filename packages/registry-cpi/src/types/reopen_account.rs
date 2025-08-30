use {
    crate::state::{discriminator as DISCRIMINATOR, seed as SEED, Bump, Config, UserId},
    base::{
        accounts::{
            AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount, SystemProgram,
        },
        converters::ByteReader,
        types::Result,
    },
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountMetas)]
pub struct Accounts<'a> {
    pub system_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [system_program, sender, bump, config, user_id, user_account, user_rotation_state] =
            accounts
        else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SystemProgram::check(system_program)?;
        SignerAccount::check(sender)?;
        ProgramAccount::check::<Bump>(bump, &crate::ID, Some(&[SEED::BUMP]))?;
        ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
        ProgramAccount::check::<UserId>(user_id, &crate::ID, Some(&[SEED::USER_ID, sender.key()]))?;
        // user_account,
        // user_rotation_state,

        Ok(Self {
            system_program,
            sender,
            bump,
            config,
            user_id,
            user_account,
            user_rotation_state,
        })
    }
}

#[derive(Default)]
pub struct InstructionData {
    pub max_data_size: u32,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        ByteReader::new::<Self>(data, 0)
            .read_u32(|x| &mut x.max_data_size)?
            .complete()
            .map(|(x, _)| x)
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::converters::{ByteWriter, ByteWriterVecExt};

        let mut buffer = vec![];
        let position = ByteWriter::from_vec(&mut buffer)
            .write_u8(DISCRIMINATOR::REOPEN_ACCOUNT)?
            .write_u32(self.max_data_size)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
