use {
    crate::state::{discriminator as DISCRIMINATOR, seed as SEED, Bump, Config, UserId},
    base::{
        accounts::{AccountCheck, ProgramAccount, ProgramAccountCheck, SignerAccount},
        converters::ByteReader,
        types::{Result, Space},
    },
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

#[derive(AccountMetas)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [sender, bump, config, user_id, user_rotation_state] = accounts else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SignerAccount::check(sender)?;
        ProgramAccount::check(bump, &crate::ID, Bump::get_space(), Some(&[SEED::BUMP]))?;
        ProgramAccount::check(
            config,
            &crate::ID,
            Config::get_space(),
            Some(&[SEED::CONFIG]),
        )?;
        ProgramAccount::check(
            user_id,
            &crate::ID,
            UserId::get_space(),
            Some(&[SEED::USER_ID, sender.key()]),
        )?;
        // user_rotation_state, // TODO: should check but not here

        Ok(Self {
            sender,
            bump,
            config,
            user_id,
            user_rotation_state,
        })
    }
}

#[derive(Default)]
pub struct InstructionData {
    pub new_owner: Pubkey,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        ByteReader::new::<Self>(data, 0)
            .read_pubkey(|x| &mut x.new_owner)?
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
            .write_u8(DISCRIMINATOR::REQUEST_ACCOUNT_ROTATION)?
            .write_pubkey(&self.new_owner)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
