use {
    crate::{
        state::discriminator as DISCRIMINATOR,
        types::common::{AssetItem, Range},
    },
    base::{
        converters::{to_u32, ByteReader, ByteWriter},
        types::{Result, ZeroCopyDeserialize, ZeroCopySerialize},
    },
    macro_try_from::AccountTryFrom,
    macro_zc_serde::{ZCDeserialize, ZCSerialize},
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
    r#macro_account::AccountMetas,
};

// TODO: implement rest accounts
#[repr(C)]
#[derive(AccountMetas, AccountTryFrom)]
pub struct Accounts<'a> {
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,

    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub bump: &'a AccountInfo,

    #[account(writable)]
    pub config: &'a AccountInfo,

    #[account(writable)]
    pub user_counter: &'a AccountInfo,

    #[account(writable)]
    pub admin_rotation_state: &'a AccountInfo,

    pub revenue_mint: &'a AccountInfo,

    #[account(writable)]
    pub revenue_app_ata: &'a AccountInfo,
}

// ZCSerialize,
#[repr(C)]
#[derive(Default, ZCSerialize, ZCDeserialize)]
pub struct InstructionData {
    pub rotation_timeout: Option<u32>,
    pub account_registration_fee: Option<AssetItem>,
    pub account_data_size_range: Option<Range>,
}

// impl ZeroCopySerialize for InstructionData {
//     fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
//         ByteWriter::new(data)
//             .write_option(&self.rotation_timeout, u32_as_bytes)?
//             .write_option_custom(&self.account_registration_fee)?
//             .write_option_custom(&self.account_data_size_range)?
//             .complete()
//     }
// }

// TODO
/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::converters::{u32_as_bytes, ByteWriter, ByteWriterVecExt};

        let mut buffer = vec![];
        let position = ByteWriter::from_vec(&mut buffer)
            .write_u8(DISCRIMINATOR::INIT)?
            .write_option(&self.rotation_timeout, u32_as_bytes)?
            .write_option_custom(&self.account_registration_fee)?
            .write_option_custom(&self.account_data_size_range)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
