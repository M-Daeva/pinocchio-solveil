use {
    crate::types::common::{AssetItem, Range},
    base::{
        helpers::{get_flag, get_space, set_flag},
        types::Space,
    },
    bytemuck::{Pod, Zeroable},
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
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Bump {
    pub config: u8,
    pub user_counter: u8,
    pub rotation_state: u8,
}

impl Space for Bump {
    #[inline]
    fn get_space() -> usize {
        get_space::<Self>()
    }
}

#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Config {
    /// can update the config and execute priveledged instructions
    pub admin: Pubkey,
    pub is_paused: u8,
    pub rotation_timeout: [u8; 4],
    pub registration_fee: AssetItem,
    pub data_size_range: Range,
}

impl Config {
    const PAUSE_BIT: u8 = 0;

    #[inline]
    pub fn is_paused(&self) -> bool {
        get_flag(self.is_paused, Self::PAUSE_BIT)
    }

    #[inline]
    pub fn set_is_paused(&mut self, is_paused: bool) {
        self.is_paused = set_flag(self.is_paused, Self::PAUSE_BIT, is_paused);
    }

    #[inline]
    pub fn rotation_timeout(&self) -> u32 {
        u32::from_le_bytes(self.rotation_timeout)
    }

    #[inline]
    pub fn set_rotation_timeout(&mut self, rotation_timeout: u32) {
        self.rotation_timeout = rotation_timeout.to_le_bytes();
    }
}

impl Space for Config {
    #[inline]
    fn get_space() -> usize {
        get_space::<Self>()
    }
}

/// for indexing
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct UserCounter {
    pub last_user_id: u32,
}

impl Space for UserCounter {
    #[inline]
    fn get_space() -> usize {
        get_space::<Self>()
    }
}

/// to transfer ownership from one address to another in 2 steps (for security reasons) \
/// used both for app admin and user accounts
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct RotationState {
    pub owner: Pubkey,
    pub new_owner: Pubkey,
    pub expiration_date: [u8; 8],
}

impl RotationState {
    #[inline]
    pub fn expiration_date(&self) -> u64 {
        u64::from_le_bytes(self.expiration_date)
    }

    #[inline]
    pub fn set_expiration_date(&mut self, expiration_date: u64) {
        self.expiration_date = expiration_date.to_le_bytes();
    }
}

impl Space for RotationState {
    #[inline]
    fn get_space() -> usize {
        get_space::<Self>()
    }
}

/// get by user: Pubkey
#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct UserId {
    pub flags: u8,
    pub id: [u8; 4],
    pub account_bump: u8,
    pub rotation_state_bump: u8,
}

impl UserId {
    const IS_OPEN: u8 = 0;
    const IS_ACTIVATED: u8 = 1;

    #[inline]
    pub fn is_open(&self) -> bool {
        get_flag(self.flags, Self::IS_OPEN)
    }

    #[inline]
    pub fn set_is_open(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::IS_OPEN, flag);
    }

    #[inline]
    pub fn is_activated(&self) -> bool {
        get_flag(self.flags, Self::IS_ACTIVATED)
    }

    #[inline]
    pub fn set_is_activated(&mut self, flag: bool) {
        self.flags = set_flag(self.flags, Self::IS_ACTIVATED, flag);
    }
}

impl Space for UserId {
    #[inline]
    fn get_space() -> usize {
        get_space::<Self>()
    }
}

/// get by user_id: u32
#[derive(Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct UserAccount {
    /// encrypted user data
    pub data: [u8; ACCOUNT_DATA_SIZE_MAX as usize],
    /// encryption nonce
    pub nonce: [u8; 8],
    /// allocated storage capacity
    max_size: [u8; 4],
}

impl Default for UserAccount {
    #[inline]
    fn default() -> Self {
        Self {
            data: [0; ACCOUNT_DATA_SIZE_MAX as usize],
            nonce: [0; 8],
            max_size: [0; 4],
        }
    }
}

impl UserAccount {
    #[inline]
    pub fn max_size(&self) -> u32 {
        u32::from_le_bytes(self.max_size)
    }

    #[inline]
    pub fn set_max_size(&mut self, max_size: u32) {
        self.max_size = max_size.to_le_bytes();
    }
}

impl UserAccount {
    #[inline]
    pub fn get_space(_max_size: u32) -> usize {
        get_space::<Self>()
    }

    // pub fn get_space(_max_size: u32) -> usize {
    //     // ACCOUNT_DATA_SIZE_MAX
    //     const DATA: usize = 4_096;
    //     // u64
    //     const NONCE: usize = 8;
    //     // u32
    //     const MAX_SIZE: usize = 4;

    //     DATA + NONCE + MAX_SIZE
    // }
}
