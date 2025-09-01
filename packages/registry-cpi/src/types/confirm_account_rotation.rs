use {
    crate::state::discriminator as DISCRIMINATOR,
    base::{
        converters::{ByteReader, ByteWriter},
        types::{Result, ZeroCopyDeserialize, ZeroCopySerialize},
    },
    macro_try_from::AccountTryFrom,
    macro_zc_serde::{ZCDeserialize, ZCSerialize},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    r#macro_account::AccountMetas,
};

#[repr(C)]
#[derive(AccountMetas, AccountTryFrom)]
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

#[repr(C)]
#[derive(Default, ZCSerialize, ZCDeserialize)]
pub struct InstructionData {}

// impl TryFrom<&[u8]> for InstructionData {
//     type Error = ProgramError;

//     fn try_from(data: &[u8]) -> Result<Self> {
//         ByteReader::new::<Self>(data, 0).complete().map(|(x, _)| x)
//     }
// }

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
