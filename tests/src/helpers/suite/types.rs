use {
    crate::helpers::suite::decimal::{str_to_dec, Decimal},
    counter_cpi::error::AnyError,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    std::fmt::Debug,
    strum::IntoEnumIterator,
    strum_macros::{Display, EnumIter, IntoStaticStr},
};

const SOL_AMOUNT_DEFAULT: u64 = 1_000;
const SOL_AMOUNT_INCREASED: u64 = 100_000;

const TOKEN_AMOUNT_DEFAULT: u64 = 1_000_000_000;
const TOKEN_AMOUNT_INCREASED: u64 = 100_000_000_000;

const DECIMALS_COIN_SOL: u8 = 9;
const DECIMALS_TOKEN_DEFAULT: u8 = 6;
const DECIMALS_TOKEN_WBTC: u8 = 8;

const PRICE_COIN_SOL: &str = "100";
const PRICE_TOKEN_USDC: &str = "1";
const PRICE_TOKEN_PYTH: &str = "0.1";
const PRICE_TOKEN_WBTC: &str = "100000";

const KEYPAIR_ADMIN: &str =
    "3SKiuW2cbAJH8KDAuhB5cdJnAGU8Y9a95gRWMFB6zPy8XH45HTNebRALhL1EqPv2QkBytb8iTu577TcmLutkzC9g";
const PUBKEY_ADMIN: &str = "Fk5wpZL2pV8AYkMKnEo5TAJ1p88FmUxBbKsZLwpiqWqQ";

const KEYPAIR_ALICE: &str =
    "4TwYiTAG6eHLznaSGZinmQGSFxKxxmx7DHwKcbs5WkasMmLPP5fv1BYKJjsfmR47KFzmA2gs5DHtsZnR8YvMCinB";
const PUBKEY_ALICE: &str = "68ZZmGRDn5971SDrj5Ldj6MUJTeRUdSV1NQUuzsaQ4N3";

const KEYPAIR_BOB: &str =
    "zsbe2oRXt1K3gRNCurjZFTVzQtYqJjhyPAQMk4VsLWe3QoU3pMGZDVVRvmgZXgLtXvAsC9kGi4ShpYpjrQbtaf5";
const PUBKEY_BOB: &str = "FPS369ZvUkQTdsU8pzmypafNnNghiDHi8G6gDCvux5SB";

const KEYPAIR_USDC: &str =
    "5gi185z4U57MEkJJTzweNrJQJQftaQH2onL8ZGRyXKWA3zspJWyQfF1J8ZRV7zd3D8aZyZxtaw8MsPZpMLMGh6L2";
const PUBKEY_USDC: &str = "8XdLEJXrM3yYfFg5EpMqcCKmXXSQeBKfTjvNP619LbE2";

const KEYPAIR_PYTH: &str =
    "5fbcPBxRADG5oxsK3K7PtM5A2CXFSErQ7bWoTXA1qeZsngyFYzWKUm4R7pBtD9fazVA9FgFC4h4WschCTQ7xjeJG";
const PUBKEY_PYTH: &str = "HBtzyBH14hR6t5UfYT3ptL6d1pMnVCep2RY8vUgHmaRA";

const KEYPAIR_WBTC: &str =
    "2RyN2wrHo8fDrvqULn61ThcSeMyBE3eQ35ADxk5bvjkMrtZRKZwYNRQgxS33UkTrw3udySYMeoJxapbLbyz3aDiZ";
const PUBKEY_WBTC: &str = "An6eCPnnsspFAy5bUrgnNkU4hkedv9ZDRUJazUTG1ewb";

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter, PartialEq)]
pub enum AppUser {
    Admin,
    Alice,
    Bob,
}

impl AppUser {
    pub fn pubkey(&self) -> Pubkey {
        let str_const = match self {
            Self::Admin => PUBKEY_ADMIN,
            Self::Alice => PUBKEY_ALICE,
            Self::Bob => PUBKEY_BOB,
        };

        Pubkey::from_str_const(str_const)
    }

    pub fn keypair(&self) -> Keypair {
        let base58_string = match self {
            Self::Admin => KEYPAIR_ADMIN,
            Self::Alice => KEYPAIR_ALICE,
            Self::Bob => KEYPAIR_BOB,
        };

        Keypair::from_base58_string(base58_string)
    }

    pub fn list() {
        for item in Self::iter() {
            println!("{:#?}: {:#?}", item, item.pubkey());
        }
        println!();
    }

    pub fn get_initial_asset_amount(&self, asset: impl Into<AppAsset>) -> u64 {
        match self {
            Self::Admin => match asset.into() {
                AppAsset::Coin(_) => SOL_AMOUNT_INCREASED,
                AppAsset::Token(_) => TOKEN_AMOUNT_INCREASED,
            },
            _ => match asset.into() {
                AppAsset::Coin(_) => SOL_AMOUNT_DEFAULT,
                AppAsset::Token(_) => TOKEN_AMOUNT_DEFAULT,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter, PartialEq)]
pub enum AppCoin {
    SOL,
}

#[derive(Debug, Clone, Copy, Display, IntoStaticStr, EnumIter, PartialEq)]
pub enum AppToken {
    USDC,
    PYTH,
    WBTC,
    WSOL,
}

impl AppToken {
    pub fn pubkey(&self) -> Pubkey {
        let str_const = match self {
            Self::USDC => PUBKEY_USDC,
            Self::PYTH => PUBKEY_PYTH,
            Self::WBTC => PUBKEY_WBTC,
            Self::WSOL => &spl_token::native_mint::ID.to_string(),
        };

        Pubkey::from_str_const(str_const)
    }

    pub fn keypair(&self) -> Keypair {
        let base58_string = match self {
            Self::USDC => KEYPAIR_USDC,
            Self::PYTH => KEYPAIR_PYTH,
            Self::WBTC => KEYPAIR_WBTC,
            Self::WSOL => panic!("WSOL doesn't have keypair!"),
        };

        Keypair::from_base58_string(base58_string)
    }

    pub fn list() {
        for item in Self::iter() {
            println!("{:#?}: {:#?}", item, item.pubkey());
        }
        println!();
    }
}

pub trait GetPrice {
    fn get_price(&self) -> Decimal;
}

impl GetPrice for AppAsset {
    fn get_price(&self) -> Decimal {
        match self {
            Self::Coin(project_coin) => project_coin.get_price(),
            Self::Token(project_token) => project_token.get_price(),
        }
    }
}

impl GetPrice for AppCoin {
    fn get_price(&self) -> Decimal {
        let price = match self {
            Self::SOL => PRICE_COIN_SOL,
        };

        str_to_dec(price)
    }
}

impl GetPrice for AppToken {
    fn get_price(&self) -> Decimal {
        let price = match self {
            Self::USDC => PRICE_TOKEN_USDC,
            Self::PYTH => PRICE_TOKEN_PYTH,
            Self::WBTC => PRICE_TOKEN_WBTC,
            Self::WSOL => PRICE_COIN_SOL,
        };

        str_to_dec(price)
    }
}

pub trait GetDecimals {
    fn get_decimals(&self) -> u8;
}

impl GetDecimals for AppAsset {
    fn get_decimals(&self) -> u8 {
        match self {
            Self::Coin(project_coin) => project_coin.get_decimals(),
            Self::Token(project_token) => project_token.get_decimals(),
        }
    }
}

impl GetDecimals for AppCoin {
    fn get_decimals(&self) -> u8 {
        match self {
            Self::SOL => DECIMALS_COIN_SOL,
        }
    }
}

impl GetDecimals for AppToken {
    fn get_decimals(&self) -> u8 {
        match self {
            Self::USDC => DECIMALS_TOKEN_DEFAULT,
            Self::PYTH => DECIMALS_TOKEN_DEFAULT,
            Self::WBTC => DECIMALS_TOKEN_WBTC,
            Self::WSOL => DECIMALS_COIN_SOL,
        }
    }
}

#[derive(Debug, Clone, Copy, Display)]
pub enum AppAsset {
    Coin(AppCoin),
    Token(AppToken),
}

impl From<AppCoin> for AppAsset {
    fn from(project_coin: AppCoin) -> Self {
        Self::Coin(project_coin)
    }
}

impl From<AppToken> for AppAsset {
    fn from(project_token: AppToken) -> Self {
        Self::Token(project_token)
    }
}

pub type TestResult<T> = std::result::Result<T, TestError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestError {
    pub info: String,
    pub index: Option<u32>,
}

impl TestError {
    pub fn from_any_error(any_error: AnyError, index: u32) -> Self {
        Self {
            info: format!("{:?}", any_error),
            index: Some(index),
        }
    }

    pub fn from_unknown(message: impl ToString) -> Self {
        Self {
            info: message.to_string(),
            index: None,
        }
    }

    pub fn from_raw_error(error: impl Debug) -> Self {
        Self {
            info: format!("{:?}", error),
            index: None,
        }
    }

    pub fn get_err_by_idx(idx: u32) -> Option<AnyError> {
        AnyError::all().find(|&x| x.index() == idx)
    }

    /// Parses custom program error from a vector of log strings
    /// Returns the error code as u32 if found, otherwise returns None
    pub fn parse_custom_program_error(logs: &[String]) -> Option<u32> {
        const ERROR_PREFIX: &str = "failed: custom program error: ";

        for log in logs {
            if let Some(error_start) = log.find(ERROR_PREFIX) {
                let error_part = &log[error_start + ERROR_PREFIX.len()..];

                // Find the hex value (should start with "0x")
                if error_part.starts_with("0x") {
                    let hex_str = &error_part[2..]; // Remove "0x" prefix

                    // Parse hex string to u32
                    if let Ok(error_code) = u32::from_str_radix(hex_str, 16) {
                        return Some(error_code);
                    }
                }
            }
        }

        None
    }

    pub fn parse_program_error(logs: &[String]) -> Option<&str> {
        const ERROR_PREFIX: &str = "failed: ";

        for log in logs {
            if let Some(error_start) = log.find(ERROR_PREFIX) {
                let error_part = &log[error_start + ERROR_PREFIX.len()..];

                return Some(error_part);
            }
        }

        None
    }
}

// #[derive(Default, Debug)]
// pub struct AccountsBuilder {
//     list: Vec<AccountMeta>,
// }

// impl AccountsBuilder {
//     /// accepts 1st keypair as payer
//     pub fn new(signers: &[Keypair]) -> Self {
//         if signers.is_empty() {
//             Self::default()
//         } else {
//             let payer = signers[0].pubkey();

//             Self {
//                 list: vec![AccountMeta::new(payer, true)],
//             }
//         }
//     }

//     pub fn with_signer(mut self, pubkey: Pubkey) -> Self {
//         self.list.push(AccountMeta::new(pubkey, true));
//         self
//     }

//     pub fn with(mut self, pubkey: Pubkey) -> Self {
//         self.list.push(AccountMeta::new(pubkey, false));
//         self
//     }

//     pub fn with_readonly(mut self, pubkey: Pubkey) -> Self {
//         self.list.push(AccountMeta::new_readonly(pubkey, false));
//         self
//     }

//     pub fn build(self) -> Vec<AccountMeta> {
//         self.list
//     }
// }
