use {
    crate::state::{discriminator as DISCRIMINATOR, seed as SEED, UserId},
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        converters::ByteReader,
        types::Result,
    },
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountMetas)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [sender, user_id, user_account] = accounts else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SignerAccount::check(sender)?;
        ProgramAccount::check::<UserId>(user_id, &crate::ID, Some(&[SEED::USER_ID, sender.key()]))?;
        // user_account, // TODO: should check but not here

        Ok(Self {
            sender,
            user_id,
            user_account,
        })
    }
}

#[derive(Default)]
pub struct InstructionData {
    pub data: String,
    pub nonce: u64,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        ByteReader::new::<Self>(data, 0)
            .read_string(|x| &mut x.data)?
            .read_u64(|x| &mut x.nonce)?
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
            .write_u8(DISCRIMINATOR::WRITE_DATA)?
            .write_string(&self.data)?
            .write_u64(self.nonce)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
