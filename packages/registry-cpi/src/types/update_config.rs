use {
    crate::{state::discriminator as DISCRIMINATOR, types::common::Range},
    base::{
        converters::{to_bool, to_pubkey, to_u32, to_u64, ByteReader},
        types::{Result, ZeroCopyDeserialize},
    },
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

#[repr(C)]
#[derive(AccountMetas, AccountTryFrom)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub config: &'a AccountInfo,

    #[account(writable)]
    pub admin_rotation_state: &'a AccountInfo,
}

#[repr(C)]
#[derive(Default)]
pub struct InstructionData {
    pub admin: Option<Pubkey>,
    pub is_paused: Option<bool>,
    pub rotation_timeout: Option<u32>,
    pub registration_fee_amount: Option<u64>,
    pub data_size_range: Option<Range>,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        ByteReader::new::<Self>(data, 0)
            .read_option(|x| &mut x.admin, to_pubkey)?
            .read_option(|x| &mut x.is_paused, to_bool)?
            .read_option(|x| &mut x.rotation_timeout, to_u32)?
            .read_option(|x| &mut x.registration_fee_amount, to_u64)?
            .read_option(|x| &mut x.data_size_range, Range::deserialize_from)?
            .complete()
            .map(|(x, _)| x)
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::converters::{
            bool_as_bytes, pubkey_as_bytes, u32_as_bytes, u64_as_bytes, ByteWriter,
            ByteWriterVecExt,
        };

        let mut buffer = vec![];
        let position = ByteWriter::from_vec(&mut buffer)
            .write_u8(DISCRIMINATOR::UPDATE_CONFIG)?
            .write_option(&self.admin, pubkey_as_bytes)?
            .write_option(&self.is_paused, bool_as_bytes)?
            .write_option(&self.rotation_timeout, u32_as_bytes)?
            .write_option(&self.registration_fee_amount, u64_as_bytes)?
            .write_option_custom(&self.data_size_range)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
