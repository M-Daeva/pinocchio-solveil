use {
    crate::{converters::to_u64, guards::check_derived_pda, types::Result},
    pinocchio::{
        account_info::AccountInfo,
        instruction::{Seed, Signer},
        pubkey::{find_program_address, Pubkey},
        sysvars::{clock::Clock, rent::Rent, Sysvar},
        ProgramResult,
    },
};

/// pass args in the order to expect mint_a <= mint_b
#[inline]
pub fn are_mints_sorted(mint_a: &Pubkey, mint_b: &Pubkey) -> bool {
    mint_a <= mint_b
}

#[inline]
pub fn sort_mints(mint_a: &Pubkey, mint_b: &Pubkey) -> (Pubkey, Pubkey) {
    if are_mints_sorted(mint_a, mint_b) {
        (*mint_a, *mint_b)
    } else {
        (*mint_b, *mint_a)
    }
}

#[inline]
pub fn get_flag(field: u8, bit: u8) -> bool {
    (field & 1 << bit) >> bit == 1
}

#[inline]
pub fn set_flag(field: u8, bit: u8, flag: bool) -> u8 {
    if flag {
        set_bit(field, bit)
    } else {
        reset_bit(field, bit)
    }
}

#[inline]
fn set_bit(field: u8, bit: u8) -> u8 {
    field | 1 << bit
}

#[inline]
fn reset_bit(field: u8, bit: u8) -> u8 {
    field & !(1 << bit)
}

#[inline]
pub fn get_clock_time() -> Result<u64> {
    Ok(Clock::get()?.unix_timestamp as u64)
}

#[inline]
pub fn get_space<T>() -> usize {
    core::mem::size_of::<T>()
}

#[inline]
pub fn get_rent_exempt(space: usize) -> Result<u64> {
    Ok(Rent::get()?.minimum_balance(space))
}

#[inline]
pub fn get_and_check_pda(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    account: Option<&AccountInfo>,
) -> Result<(Pubkey, u8)> {
    let (pda, bump) = find_program_address(seeds, program_id);

    if let Some(target_account) = account {
        check_derived_pda(&pda, target_account)?;
    }

    Ok((pda, bump))
}

#[inline]
pub fn create_account_with_signer(
    payer: &AccountInfo,
    account: &AccountInfo,
    space: usize,
    signer_seeds: &[Seed],
    owner: &Pubkey,
) -> ProgramResult {
    let lamports = get_rent_exempt(space)?;
    let signers = &[Signer::from(signer_seeds)];

    (pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: account,
        lamports,
        space: space as u64,
        owner,
    })
    .invoke_signed(signers)
}

#[inline]
pub fn create_account(
    payer: &AccountInfo,
    account: &AccountInfo,
    space: usize,
    owner: &Pubkey,
) -> ProgramResult {
    let lamports = get_rent_exempt(space)?;

    (pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: account,
        lamports,
        space: space as u64,
        owner,
    })
    .invoke()
}

#[inline]
pub fn init_mint_account(
    account: &AccountInfo,
    decimals: u8,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
) -> ProgramResult {
    pinocchio_token::instructions::InitializeMint2 {
        mint: account,
        decimals,
        mint_authority,
        freeze_authority,
    }
    .invoke()
}

#[inline]
pub fn init_token_account(
    account: &AccountInfo,
    mint: &AccountInfo,
    owner: &Pubkey,
) -> ProgramResult {
    pinocchio_token::instructions::InitializeAccount3 {
        account,
        mint,
        owner,
    }
    .invoke()
}

#[inline]
pub fn create_ata(
    payer: &AccountInfo,
    account: &AccountInfo,
    mint: &AccountInfo,
    owner: &AccountInfo,
    system_program: &AccountInfo,
    token_program: &AccountInfo,
) -> ProgramResult {
    pinocchio_associated_token_account::instructions::Create {
        funding_account: payer,
        account,
        wallet: owner,
        mint,
        system_program,
        token_program,
    }
    .invoke()
}

#[inline]
pub fn transfer_sol_from_user(amount: u64, from: &AccountInfo, to: &AccountInfo) -> ProgramResult {
    pinocchio_system::instructions::Transfer {
        from,
        to,
        lamports: amount,
    }
    .invoke()
}

#[inline]
pub fn transfer_sol_from_program(
    amount: u64,
    from: &AccountInfo,
    to: &AccountInfo,
    signer_seeds: &[Seed],
) -> ProgramResult {
    let signers = &[Signer::from(signer_seeds)];

    pinocchio_system::instructions::Transfer {
        from,
        to,
        lamports: amount,
    }
    .invoke_signed(signers)
}

#[inline]
pub fn transfer_token_from_user(
    amount: u64,
    mint: &AccountInfo,
    from: &AccountInfo,
    to: &AccountInfo,
    authority: &AccountInfo,
    decimals: u8,
) -> ProgramResult {
    pinocchio_token::instructions::TransferChecked {
        amount,
        mint,
        from,
        to,
        authority,
        decimals,
    }
    .invoke()
}

#[allow(clippy::too_many_arguments)]
#[inline]
pub fn transfer_token_from_program(
    amount: u64,
    mint: &AccountInfo,
    from: &AccountInfo,
    to: &AccountInfo,
    signer_seeds: &[Seed],
    authority: &AccountInfo,
    decimals: u8,
) -> ProgramResult {
    let signers = &[Signer::from(signer_seeds)];

    pinocchio_token::instructions::TransferChecked {
        amount,
        mint,
        from,
        to,
        authority,
        decimals,
    }
    .invoke_signed(signers)
}

#[inline]
pub fn mint_token_to(
    amount: u64,
    mint: &AccountInfo,
    to: &AccountInfo,
    signer_seeds: &[Seed],
    authority: &AccountInfo,
    decimals: u8,
) -> ProgramResult {
    let signers = &[Signer::from(signer_seeds)];

    pinocchio_token::instructions::MintToChecked {
        mint,
        account: to,
        mint_authority: authority,
        amount,
        decimals,
    }
    .invoke_signed(signers)
}

#[inline]
pub fn burn_token_from(
    amount: u64,
    mint: &AccountInfo,
    from: &AccountInfo,
    authority: &AccountInfo,
    decimals: u8,
) -> ProgramResult {
    pinocchio_token::instructions::BurnChecked {
        account: from,
        mint,
        authority,
        amount,
        decimals,
    }
    .invoke()
}

#[inline]
pub fn get_ata_balance(ata: &AccountInfo) -> Result<u64> {
    // Token account data layout (SPL Token format):
    // - mint: 32 bytes
    // - owner: 32 bytes
    // - amount: 8 bytes (offset 64)
    // - delegate: 36 bytes
    // - state: 1 byte
    // - is_native: 12 bytes
    // - delegated_amount: 8 bytes
    // - close_authority: 36 bytes
    const INDEX_START: usize = 64;
    // const INDEX_END: usize = 72;

    let data = &ata.try_borrow_data()?;
    // let amount_bytes = &data[INDEX_START..INDEX_END];

    to_u64(data, INDEX_START).map(|(x, _)| x)
}

#[inline]
pub fn get_token_decimals(mint: &AccountInfo) -> Result<u8> {
    // Mint account data layout:
    // - mint_authority: 36 bytes (32 + 4 for COption)
    // - supply: 8 bytes
    // - decimals: 1 byte (offset 44)
    // - is_initialized: 1 byte
    // - freeze_authority: 36 bytes
    const INDEX_START: usize = 44;

    Ok(mint.try_borrow_data()?[INDEX_START])
}
