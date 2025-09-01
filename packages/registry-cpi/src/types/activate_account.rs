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
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    #[account(writable)]
    pub user_id: &'a AccountInfo,

    pub revenue_mint: &'a AccountInfo,

    #[account(writable)]
    pub revenue_sender_ata: &'a AccountInfo,

    #[account(writable)]
    pub revenue_app_ata: &'a AccountInfo,
}

#[repr(C)]
#[derive(Default, ZCDeserialize)]
#[test_serialize(DISCRIMINATOR::ACTIVATE_ACCOUNT)]
pub struct InstructionData {
    pub user: Pubkey,
}

// impl TryFrom<&[u8]> for InstructionData {
//     type Error = ProgramError;

//     fn try_from(data: &[u8]) -> Result<Self> {
//         ByteReader::new::<Self>(data, 0)
//             .read_pubkey(|x| &mut x.user)?
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
//             .write_u8(DISCRIMINATOR::ACTIVATE_ACCOUNT)?
//             .write_pubkey(&self.user)?
//             .position();
//         buffer.truncate(position);

//         Ok(buffer)
//     }
// }
