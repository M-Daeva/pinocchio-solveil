use {
    crate::state::{seed as SEED, Bump, Config},
    base::{
        accounts::{
            AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, MintAccount,
            ProgramAccount, ProgramAccountCheck, SignerAccount, SystemProgram,
        },
        converters::{to_u64, ByteReader},
        types::{Result, Space},
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

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [system_program, token_program, associated_token_program, sender, recipient, bump, config, revenue_mint, revenue_recipient_ata, revenue_app_ata] =
            accounts
        else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        SystemProgram::check(system_program)?;
        // token_program,
        // associated_token_program,
        SignerAccount::check(sender)?;
        // recipient,
        ProgramAccount::check(bump, &crate::ID, Bump::get_space(), Some(&[SEED::BUMP]))?;
        ProgramAccount::check(
            config,
            &crate::ID,
            Config::get_space(),
            Some(&[SEED::CONFIG]),
        )?;
        MintAccount::check(revenue_mint)?;
        AssociatedTokenAccount::check(
            revenue_recipient_ata,
            recipient,
            revenue_mint,
            token_program,
        )?;
        AssociatedTokenAccount::check(revenue_app_ata, config, revenue_mint, token_program)?;

        Ok(Self {
            system_program,
            token_program,
            associated_token_program,
            sender,
            recipient,
            bump,
            config,
            revenue_mint,
            revenue_recipient_ata,
            revenue_app_ata,
        })
    }
}

#[derive(Default)]
pub struct InstructionData {
    pub amount: Option<u64>,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        ByteReader::new::<Self>(data, 0)
            .read_option(|x| &mut x.amount, to_u64)?
            .complete()
            .map(|(x, _)| x)
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::converters::{u64_as_bytes, ByteWriter, ByteWriterVecExt};

        let mut buffer = vec![];
        let position = ByteWriter::from_vec(&mut buffer)
            .write_u8(crate::state::discriminator::INIT)?
            .write_option(&self.amount, u64_as_bytes)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
