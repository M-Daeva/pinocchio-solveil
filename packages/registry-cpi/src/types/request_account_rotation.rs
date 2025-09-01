use {
    crate::state::discriminator as DISCRIMINATOR,
    base::{
        converters::ByteReader,
        types::{Result, ZeroCopyDeserialize},
    },
    macro_test_ser::test_serialize,
    macro_try_from::AccountTryFrom,
    macro_zc_serde::ZCDeserialize,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

#[repr(C)]
#[derive(AccountTryFrom, AccountMetas)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    pub user_id: &'a AccountInfo,

    #[account(writable)]
    pub user_rotation_state: &'a AccountInfo,
}

#[repr(C)]
#[derive(Default, ZCDeserialize)]
#[test_serialize(DISCRIMINATOR::REQUEST_ACCOUNT_ROTATION)]
pub struct InstructionData {
    pub new_owner: Pubkey,
}

// impl TryFrom<&[u8]> for InstructionData {
//     type Error = ProgramError;

//     fn try_from(data: &[u8]) -> Result<Self> {
//         ByteReader::new::<Self>(data, 0)
//             .read_pubkey(|x| &mut x.new_owner)?
//             .complete()
//             .map(|(x, _)| x)
//     }
// }

// /// for tests
// #[cfg(feature = "dev")]
// impl base::types::InstructionSerialize for InstructionData {
//     fn serialize(&self) -> Result<Vec<u8>> {
//         use base::converters::{ByteWriter, ByteWriterVecExt};

//         let mut buffer = vec![];
//         let position = ByteWriter::from_vec(&mut buffer)
//             .write_u8(DISCRIMINATOR::REQUEST_ACCOUNT_ROTATION)?
//             .write_pubkey(&self.new_owner)?
//             .position();
//         buffer.truncate(position);

//         Ok(buffer)
//     }
// }
