use {
    crate::types::common::{AssetItem, Range},
    base::{
        traits::DataLen,
        types::{BitField, Uint32, Uint64},
    },
    bytemuck::{Pod, Zeroable},
    codama::CodamaAccount,
    macro_optional_flag::OptionFlag,
    macro_p_serde::p_serde,
    pinocchio::pubkey::Pubkey,
    pinocchio_pubkey::pubkey,
};

#[repr(u8)]
pub enum Discriminator {
    // admin
    Init,
    UpdateConfig,
    ConfirmAdminRotation,
    WithdrawRevenue,
    // user
    CreateAccount,
    CloseAccount,
    ReopenAccount,
    ActivateAccount,
    WriteData,
    RequestAccountRotation,
    ConfirmAccountRotation,
}

pub mod seed {
    pub const BUMP: &[u8] = b"bump";
    pub const CONFIG: &[u8] = b"config";
    pub const USER_COUNTER: &[u8] = b"user_counter";
    pub const ADMIN_ROTATION_STATE: &[u8] = b"admin_rotation_state";

    pub const USER_ID: &[u8] = b"user_id";
    pub const USER_ACCOUNT: &[u8] = b"user_account";
    pub const USER_ROTATION_STATE: &[u8] = b"user_rotation_state";
}

pub const SECONDS_PER_DAY: u32 = 24 * 3_600;
pub const SECONDS_PER_YEAR: u32 = 365 * SECONDS_PER_DAY;

pub const CLOCK_TIME_MIN: u64 = 1750000000;
pub const MAINNET_ADMIN: Pubkey = pubkey!("AH9JvTDAiQy2zAuFfzteNyUrW5DYoTsTLoeNjXrxTTSt");

pub const ROTATION_TIMEOUT: u32 = SECONDS_PER_DAY;
pub const ACCOUNT_REGISTRATION_FEE_AMOUNT: u64 = 10_000_000; // 10 $
pub const ACCOUNT_REGISTRATION_FEE_ASSET: Pubkey =
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // mainnet USDC
pub const ACCOUNT_DATA_SIZE_MIN: u32 = 100;
/// "Account data size realloc limited to 10240 in inner instructions"
pub const ACCOUNT_DATA_SIZE_MAX: u32 = 4096;

/// to store bumps for all app accounts
#[derive(CodamaAccount)]
#[p_serde]
pub struct Bump {
    pub config: u8,
    pub user_counter: u8,
    pub rotation_state: u8,
}

#[derive(CodamaAccount)]
#[p_serde]
pub struct Config {
    /// can update the config and execute priveledged instructions
    pub admin: Pubkey,
    pub is_paused: BitField,
    pub rotation_timeout: Uint32,
    pub registration_fee: AssetItem,
    pub data_size_range: Range,
}

/// for indexing
#[derive(CodamaAccount)]
#[p_serde]
pub struct UserCounter {
    pub last_user_id: Uint32,
}

/// to transfer ownership from one address to another in 2 steps (for security reasons) \
/// used both for app admin and user accounts
#[derive(CodamaAccount)]
#[p_serde]
pub struct RotationState {
    pub owner: Pubkey,
    pub new_owner: Pubkey,
    pub expiration_date: Uint64,
}

/// get by user: Pubkey
#[derive(CodamaAccount, OptionFlag)]
#[p_serde]
pub struct UserId {
    #[optional(is_open, is_activated)]
    pub flags: BitField,
    pub id: Uint32,
    pub account_bump: u8,
    pub rotation_state_bump: u8,
}

/// get by user_id: u32
#[derive(CodamaAccount, Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct UserAccount {
    /// encrypted user data
    pub data: [u8; 4_096], // TODO: codama can't recognize `pub data: [u8; ACCOUNT_DATA_SIZE_MAX as usize]`
    /// encryption nonce
    pub nonce: Uint64,
    /// allocated storage capacity
    pub max_size: Uint32,
}

impl Default for UserAccount {
    #[inline]
    fn default() -> Self {
        Self {
            data: [0; ACCOUNT_DATA_SIZE_MAX as usize],
            nonce: Uint64::default(),
            max_size: Uint32::default(),
        }
    }
}

impl DataLen for UserAccount {
    const LEN: usize = core::mem::size_of::<Self>();
}
