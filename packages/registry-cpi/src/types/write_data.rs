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
#[derive(AccountTryFrom, AccountMetas)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,
}

#[repr(C)]
#[derive(Default, ZCSerialize, ZCDeserialize)]
pub struct InstructionData {
    pub data: String,
    pub nonce: u64,
}

// impl TryFrom<&[u8]> for InstructionData {
//     type Error = ProgramError;

//     fn try_from(data: &[u8]) -> Result<Self> {
//         ByteReader::new::<Self>(data, 0)
//             .read_string(|x| &mut x.data)?
//             .read_u64(|x| &mut x.nonce)?
//             .complete()
//             .map(|(x, _)| x)
//     }
// }

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
