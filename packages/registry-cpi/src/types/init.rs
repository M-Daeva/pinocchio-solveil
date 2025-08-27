use {
    crate::{
        state::Bump,
        types::common::{AssetItem, Range},
    },
    base::{
        accounts::{AccountCheck, MintAccount, SignerAccount, SystemProgram},
        converters::{to_option, to_u32},
        guards::check_ix_data_len,
        types::{Result, ZeroCopyDeserialize},
    },
    pinocchio::{account_info::AccountInfo, program_error::ProgramError},
    r#macro_account::AccountMetas,
};

#[derive(AccountMetas)]
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
    pub revenue_app_ata: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [system_program, token_program, associated_token_program, sender, bump, config, user_counter, admin_rotation_state, revenue_mint, revenue_app_ata] =
            accounts
        else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SystemProgram::check(system_program)?;
        // token_program
        // associated_token_program

        SignerAccount::check(sender)?;
        // bump
        // config
        // user_counter
        // admin_rotation_state
        MintAccount::check(revenue_mint)?;
        // revenue_app_ata

        Ok(Self {
            system_program,
            token_program,
            associated_token_program,
            sender,
            bump,
            config,
            user_counter,
            admin_rotation_state,
            revenue_mint,
            revenue_app_ata,
        })
    }
}

pub struct InstructionData {
    pub bumps: Bump,
    pub rotation_timeout: Option<u32>,
    pub account_registration_fee: Option<AssetItem>,
    pub account_data_size_range: Option<Range>,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        let (bumps, end_index) = Bump::deserialize_from(data, 0)?;
        let (rotation_timeout, end_index) = to_option(data, end_index, to_u32)?;
        let (account_registration_fee, end_index) =
            to_option(data, end_index, AssetItem::deserialize_from)?;
        let (account_data_size_range, end_index) =
            to_option(data, end_index, Range::deserialize_from)?;
        check_ix_data_len(data, end_index)?;

        Ok(Self {
            bumps,
            rotation_timeout,
            account_registration_fee,
            account_data_size_range,
        })
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::ZeroCopySerialize for InstructionData {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        use base::converters::{option_as_bytes, u32_as_bytes, ByteWriter};

        let mut writer = ByteWriter::new(data);
        writer.write_custom(&self.bumps)?;
        writer.write_bytes(&option_as_bytes(&self.rotation_timeout, u32_as_bytes))?;
        writer.write_option_custom(&self.account_registration_fee)?;
        writer.write_option_custom(&self.account_data_size_range)?;

        Ok(())
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::types::ZeroCopySerialize;

        let buffer: &mut [u8] = &mut [crate::state::discriminator::INIT];
        self.serialize_into(buffer)?;

        Ok(buffer.to_vec())
    }
}
