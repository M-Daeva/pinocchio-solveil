use {
    crate::state::discriminator as DISCRIMINATOR,
    base::{
        accounts::{AccountCheck, SignerAccount, SystemProgram},
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

    #[account(writable)]
    pub user_id_pre: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [system_program, sender, user_id_pre, user_id, user_rotation_state] = accounts else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SystemProgram::check(system_program)?;
        SignerAccount::check(sender)?;
        // user_id_pre,
        // ProgramAccount::check(
        //     user_id,
        //     &crate::ID,
        //     UserId::get_space(),
        //     Some(&[SEED::USER_ID, sender.key()]),
        // )?;
        // user_rotation_state, // TODO: should check but not here

        Ok(Self {
            system_program,
            sender,
            user_id_pre,
            user_id,
            user_rotation_state,
        })
    }
}

#[derive(Default)]
pub struct InstructionData {}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        ByteReader::new::<Self>(data, 0).complete().map(|(x, _)| x)
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::converters::{ByteWriter, ByteWriterVecExt};

        let mut buffer = vec![];
        let position = ByteWriter::from_vec(&mut buffer)
            .write_u8(DISCRIMINATOR::CONFIRM_ACCOUNT_ROTATION)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
