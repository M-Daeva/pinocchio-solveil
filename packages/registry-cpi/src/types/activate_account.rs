use {
    crate::state::{discriminator as DISCRIMINATOR, seed as SEED, Bump, Config},
    base::{
        accounts::{
            AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, MintAccount,
            ProgramAccount, ProgramAccountCheck, SignerAccount, SystemProgram,
        },
        converters::ByteReader,
        types::Result,
    },
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

#[repr(C)]
#[derive(AccountMetas)]
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

impl<'a> TryFrom<&'a [AccountInfo]> for Accounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self> {
        let [system_program, token_program, associated_token_program, sender, bump, config, user_id, revenue_mint, revenue_sender_ata, revenue_app_ata] =
            accounts
        else {
            Err(ProgramError::NotEnoughAccountKeys)?
        };

        // TODO: move guards to instruction handler
        SystemProgram::check(system_program)?;
        // token_program,
        // associated_token_program,
        SignerAccount::check(sender)?;
        ProgramAccount::check::<Bump>(bump, &crate::ID, Some(&[SEED::BUMP]))?;
        ProgramAccount::check::<Config>(config, &crate::ID, Some(&[SEED::CONFIG]))?;
        // ProgramAccount::check( // TODO: check with user instead of sender
        //     user_id,
        //     &crate::ID,
        //     UserId::get_space(),
        //     Some(&[SEED::USER_ID, sender.key()]),
        // )?;
        MintAccount::check(revenue_mint)?;
        AssociatedTokenAccount::check(revenue_sender_ata, sender, revenue_mint, token_program)?;
        AssociatedTokenAccount::check(revenue_app_ata, config, revenue_mint, token_program)?;

        Ok(Self {
            system_program,
            token_program,
            associated_token_program,
            sender,
            bump,
            config,
            user_id,
            revenue_mint,
            revenue_sender_ata,
            revenue_app_ata,
        })
    }
}

#[repr(C)]
#[derive(Default)]
pub struct InstructionData {
    pub user: Pubkey,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self> {
        ByteReader::new::<Self>(data, 0)
            .read_pubkey(|x| &mut x.user)?
            .complete()
            .map(|(x, _)| x)
    }
}

/// for tests
#[cfg(feature = "dev")]
impl base::types::InstructionSerialize for InstructionData {
    fn serialize(&self) -> Result<Vec<u8>> {
        use base::converters::{ByteWriter, ByteWriterVecExt};

        let mut buffer = vec![];
        let position = ByteWriter::from_vec(&mut buffer)
            .write_u8(DISCRIMINATOR::ACTIVATE_ACCOUNT)?
            .write_pubkey(&self.user)?
            .position();
        buffer.truncate(position);

        Ok(buffer)
    }
}
