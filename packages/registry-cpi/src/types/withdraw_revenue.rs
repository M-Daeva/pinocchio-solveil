use {
    base::{
        converters::{to_u64, ByteReader, ByteWriter},
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
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    // handle the option on client
    #[account(writable)]
    pub recipient: &'a AccountInfo,

    pub bump: &'a AccountInfo,

    pub config: &'a AccountInfo,

    pub revenue_mint: &'a AccountInfo,

    #[account(writable)]
    pub revenue_recipient_ata: &'a AccountInfo,

    #[account(writable)]
    pub revenue_app_ata: &'a AccountInfo,
}

#[repr(C)]
#[derive(Default, ZCSerialize, ZCDeserialize)]
pub struct InstructionData {
    pub amount: Option<u64>,
}

// impl TryFrom<&[u8]> for InstructionData {
//     type Error = ProgramError;

//     fn try_from(data: &[u8]) -> Result<Self> {
//         ByteReader::new::<Self>(data, 0)
//             .read_option(|x| &mut x.amount, to_u64)?
//             .complete()
//             .map(|(x, _)| x)
//     }
// }

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::converters::{u64_as_bytes, ByteWriter, ByteWriterVecExt};

        let mut buffer = vec![];
        let position = ByteWriter::from_vec(&mut buffer)
            .write_u8(crate::state::discriminator::WITHDRAW_REVENUE)?
            .write_option(&self.amount, u64_as_bytes)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
