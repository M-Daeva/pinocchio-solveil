use {
    crate::state::discriminator as DISCRIMINATOR,
    base::{
        converters::ByteReader,
        types::{Result, ZeroCopyDeserialize},
    },
    macro_test_ser::test_serialize,
    macro_try_from::AccountTryFrom,
    macro_zc_serde::ZCDeserialize,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[repr(C)]
#[derive(AccountTryFrom, AccountMetas)]
pub struct Accounts<'a> {
    pub system_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_account: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

#[repr(C)]
#[derive(Default, ZCDeserialize)]
#[test_serialize(DISCRIMINATOR::CLOSE_ACCOUNT)]
pub struct InstructionData {}

// impl TryFrom<&[u8]> for InstructionData {
//     type Error = ProgramError;

//     fn try_from(data: &[u8]) -> Result<Self> {
//         ByteReader::new::<Self>(data, 0).complete().map(|(x, _)| x)
//     }
// }

// /// for tests
// #[cfg(feature = "dev")]
// impl base::types::InstructionSerialize for InstructionData {
//     fn serialize(&self) -> Result<Vec<u8>> {
//         use base::converters::{ByteWriter, ByteWriterVecExt};

//         let mut buffer = vec![];
//         let position = ByteWriter::from_vec(&mut buffer)
//             .write_u8(crate::state::discriminator::CLOSE_ACCOUNT)?
//             .position();
//         buffer.truncate(position);

//         Ok(buffer)
//     }
// }
