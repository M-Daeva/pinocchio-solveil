use {
    crate::{
        guards::{
            check_account_data_len, check_account_owner, check_signer, check_system_program,
            check_token_2022_data_len,
        },
        helpers::{
            create_account, create_account_with_signer, create_ata, get_and_check_pda,
            init_mint_account, init_token_account,
        },
        types::TOKEN_2022_PROGRAM_ID,
    },
    pinocchio::{
        account_info::AccountInfo, instruction::Seed, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
};

pub trait AccountCheck {
    fn check(account: &AccountInfo) -> ProgramResult;
}

pub trait MintInit {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> ProgramResult;

    fn init_if_needed(
        payer: &AccountInfo,
        account: &AccountInfo,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> ProgramResult;
}

pub trait AssociatedTokenAccountCheck {
    fn check(
        account: &AccountInfo,
        owner: &AccountInfo,
        mint: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult;
}

pub trait AssociatedTokenAccountInit {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult;

    fn init_if_needed(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult;
}

pub trait ProgramAccountCheck {
    fn check(
        account: &AccountInfo,
        program_id: &Pubkey,
        program_account_len: u64,
        seeds: Option<&[&[u8]]>,
    ) -> ProgramResult;
}

pub trait ProgramAccountInit {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        space: u64,
        signer_seeds: &[Seed],
        owner: &Pubkey,
    ) -> ProgramResult;
}

pub trait AccountClose {
    fn close(account: &AccountInfo, destination: &AccountInfo) -> ProgramResult;
}

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> ProgramResult {
        check_signer(account)
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountInfo) -> ProgramResult {
        check_account_owner(account, &pinocchio_system::ID)
    }
}

pub struct SystemProgram;

impl AccountCheck for SystemProgram {
    fn check(account: &AccountInfo) -> ProgramResult {
        check_system_program(account)
    }
}

pub struct MintAccount;

impl AccountCheck for MintAccount {
    fn check(account: &AccountInfo) -> ProgramResult {
        check_account_owner(account, &pinocchio_token::ID)?;
        check_account_data_len(account, pinocchio_token::state::Mint::LEN)
    }
}

impl MintInit for MintAccount {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> ProgramResult {
        create_account(
            payer,
            account,
            pinocchio_token::state::Mint::LEN as u64,
            &pinocchio_token::ID,
        )?;

        init_mint_account(account, decimals, mint_authority, freeze_authority)
    }

    fn init_if_needed(
        payer: &AccountInfo,
        account: &AccountInfo,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            _ => Self::init(account, payer, decimals, mint_authority, freeze_authority),
        }
    }
}

pub struct TokenAccount;

impl AccountCheck for TokenAccount {
    fn check(account: &AccountInfo) -> ProgramResult {
        check_account_owner(account, &pinocchio_token::ID)?;
        check_account_data_len(account, pinocchio_token::state::TokenAccount::LEN)
    }
}

pub trait AccountInit {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &Pubkey,
    ) -> ProgramResult;

    fn init_if_needed(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &Pubkey,
    ) -> ProgramResult;
}

impl AccountInit for TokenAccount {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &Pubkey,
    ) -> ProgramResult {
        create_account(
            payer,
            account,
            pinocchio_token::state::TokenAccount::LEN as u64,
            &pinocchio_token::ID,
        )?;

        init_token_account(account, mint, owner)
    }

    fn init_if_needed(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &Pubkey,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            _ => Self::init(account, mint, payer, owner),
        }
    }
}

pub struct Mint2022Account;

impl AccountCheck for Mint2022Account {
    fn check(account: &AccountInfo) -> ProgramResult {
        check_account_owner(account, &TOKEN_2022_PROGRAM_ID)?;
        check_token_2022_data_len(account, pinocchio_token::state::Mint::LEN)
    }
}

impl MintInit for Mint2022Account {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> ProgramResult {
        create_account(
            payer,
            account,
            pinocchio_token::state::Mint::LEN as u64,
            &TOKEN_2022_PROGRAM_ID,
        )?;

        init_mint_account(account, decimals, mint_authority, freeze_authority)
    }

    fn init_if_needed(
        payer: &AccountInfo,
        account: &AccountInfo,
        decimals: u8,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            _ => Self::init(account, payer, decimals, mint_authority, freeze_authority),
        }
    }
}

pub struct TokenAccount2022Account;

impl AccountCheck for TokenAccount2022Account {
    fn check(account: &AccountInfo) -> ProgramResult {
        check_account_owner(account, &TOKEN_2022_PROGRAM_ID)?;
        check_token_2022_data_len(account, pinocchio_token::state::TokenAccount::LEN)
    }
}

impl AccountInit for TokenAccount2022Account {
    fn init(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &Pubkey,
    ) -> ProgramResult {
        create_account(
            payer,
            account,
            pinocchio_token::state::TokenAccount::LEN as u64,
            &TOKEN_2022_PROGRAM_ID,
        )?;

        init_token_account(account, mint, owner)
    }

    fn init_if_needed(
        account: &AccountInfo,
        mint: &AccountInfo,
        payer: &AccountInfo,
        owner: &Pubkey,
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            _ => Self::init(account, mint, payer, owner),
        }
    }
}

pub struct MintInterface;

impl AccountCheck for MintInterface {
    fn check(account: &AccountInfo) -> ProgramResult {
        if account.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return check_token_2022_data_len(account, pinocchio_token::state::Mint::LEN);
        }

        if account.is_owned_by(&pinocchio_token::ID) {
            return check_account_data_len(account, pinocchio_token::state::Mint::LEN);
        }

        Err(ProgramError::InvalidAccountOwner)
    }
}

pub struct TokenAccountInterface;

impl AccountCheck for TokenAccountInterface {
    fn check(account: &AccountInfo) -> ProgramResult {
        if account.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return check_token_2022_data_len(account, pinocchio_token::state::TokenAccount::LEN);
        }

        if account.is_owned_by(&pinocchio_token::ID) {
            return check_account_data_len(account, pinocchio_token::state::TokenAccount::LEN);
        }

        Err(ProgramError::InvalidAccountOwner)
    }
}

pub struct AssociatedTokenAccount;

impl AssociatedTokenAccountCheck for AssociatedTokenAccount {
    fn check(
        account: &AccountInfo,
        owner: &AccountInfo,
        mint: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult {
        TokenAccount::check(account)?;

        get_and_check_pda(
            &[owner.key(), token_program.key(), mint.key()],
            &pinocchio_associated_token_account::ID,
            Some(account),
        )?;

        Ok(())
    }
}

impl AssociatedTokenAccountInit for AssociatedTokenAccount {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult {
        create_ata(payer, account, mint, owner, system_program, token_program)
    }

    fn init_if_needed(
        payer: &AccountInfo,
        account: &AccountInfo,
        mint: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> ProgramResult {
        match Self::check(account, owner, mint, token_program) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(payer, account, mint, owner, system_program, token_program),
        }
    }
}

pub struct ProgramAccount;

impl ProgramAccountCheck for ProgramAccount {
    fn check(
        account: &AccountInfo,
        program_id: &Pubkey,
        program_account_len: u64,
        seeds: Option<&[&[u8]]>,
    ) -> ProgramResult {
        check_account_owner(account, program_id)?;
        check_account_data_len(account, program_account_len as usize)?;

        if let Some(seeds) = seeds {
            get_and_check_pda(seeds, program_id, Some(account))?;
        }

        Ok(())
    }
}

impl ProgramAccountInit for ProgramAccount {
    fn init(
        payer: &AccountInfo,
        account: &AccountInfo,
        space: u64,
        signer_seeds: &[Seed],
        owner: &Pubkey,
    ) -> ProgramResult {
        create_account_with_signer(payer, account, space, signer_seeds, owner)
    }

    // TODO: add init_if_needed
}

impl AccountClose for ProgramAccount {
    fn close(account: &AccountInfo, destination: &AccountInfo) -> ProgramResult {
        Self::mark_account_as_closed(account)?;
        Self::transfer_all_lamports(account, destination)?;
        Self::finalize_closure(account)
    }
}

impl ProgramAccount {
    fn mark_account_as_closed(account: &AccountInfo) -> ProgramResult {
        const CLOSED_ACCOUNT_DISCRIMINATOR: u8 = 0xFF;

        let mut account_data = account.try_borrow_mut_data()?;
        account_data[0] = CLOSED_ACCOUNT_DISCRIMINATOR;

        Ok(())
    }

    fn transfer_all_lamports(source: &AccountInfo, destination: &AccountInfo) -> ProgramResult {
        let source_lamports = source.try_borrow_lamports()?;
        let mut dest_lamports = destination.try_borrow_mut_lamports()?;

        *dest_lamports += *source_lamports;

        Ok(())
    }

    fn finalize_closure(account: &AccountInfo) -> ProgramResult {
        account.resize(1)?;
        account.close()
    }
}
