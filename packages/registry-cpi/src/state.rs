use {
    crate::types::common::{AssetItem, Range},
    base::{
        converters::{option_as_bytes, pubkey_as_bytes, to_pubkey, ByteReader, ByteWriter},
        helpers::get_space,
        types::{Result, Space, ZeroCopyDeserialize, ZeroCopySerialize},
    },
    pinocchio::{pubkey::Pubkey, ProgramResult},
    pinocchio_pubkey::pubkey,
};

pub mod discriminator {
    pub const INIT: u8 = 0;
    pub const UPDATE_CONFIG: u8 = 1;
    pub const CONFIRM_ADMIN_ROTATION: u8 = 2;
    pub const WITHDRAW_REVENUE: u8 = 3;

    pub const CREATE_ACCOUNT: u8 = 4;
    pub const CLOSE_ACCOUNT: u8 = 5;
    pub const REOPEN_ACCOUNT: u8 = 6;
    pub const ACTIVATE_ACCOUNT: u8 = 7;
    pub const WRITE_DATA: u8 = 8;
    pub const REQUEST_ACCOUNT_ROTATION: u8 = 9;
    pub const CONFIRM_ACCOUNT_ROTATION: u8 = 10;
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
pub const ACCOUNT_DATA_SIZE_MAX: u32 = 10_000;

/// to store bumps for all app accounts
#[derive(Default, Debug, PartialEq)]
pub struct Bump {
    pub config: u8,
    pub user_counter: u8,
    pub rotation_state: u8,
}

impl ZeroCopySerialize for Bump {
    fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
        ByteWriter::new(data)
            .write_u8(self.config)?
            .write_u8(self.user_counter)?
            .write_u8(self.rotation_state)?
            .complete()
    }
}

impl ZeroCopyDeserialize for Bump {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_u8(|x| &mut x.config)?
            .read_u8(|x| &mut x.user_counter)?
            .read_u8(|x| &mut x.rotation_state)?
            .complete()
    }
}

impl Space for Bump {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Config {
    /// can update the config and execute priveledged instructions
    pub admin: Pubkey,
    pub is_paused: bool,
    pub rotation_timeout: u32,
    pub registration_fee: AssetItem,
    pub data_size_range: Range,
}

impl ZeroCopySerialize for Config {
    fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
        ByteWriter::new(data)
            .write_pubkey(&self.admin)?
            .write_bool(self.is_paused)?
            .write_u32(self.rotation_timeout)?
            .write_custom(&self.registration_fee)?
            .write_custom(&self.data_size_range)?
            .complete()
    }
}

impl ZeroCopyDeserialize for Config {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_pubkey(|x| &mut x.admin)?
            .read_bool(|x| &mut x.is_paused)?
            .read_u32(|x| &mut x.rotation_timeout)?
            .read_custom(|x| &mut x.registration_fee)?
            .read_custom(|x| &mut x.data_size_range)?
            .complete()
    }
}

impl Space for Config {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// for indexing
#[derive(Default, Debug, PartialEq)]
pub struct UserCounter {
    pub last_user_id: u32,
}

impl ZeroCopySerialize for UserCounter {
    fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
        ByteWriter::new(data)
            .write_u32(self.last_user_id)?
            .complete()
    }
}

impl ZeroCopyDeserialize for UserCounter {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_u32(|x| &mut x.last_user_id)?
            .complete()
    }
}

impl Space for UserCounter {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// to transfer ownership from one address to another in 2 steps (for security reasons) \
/// used both for app admin and user accounts
#[derive(Default, Debug, PartialEq)]
pub struct RotationState {
    pub owner: Pubkey,
    pub new_owner: Option<Pubkey>,
    pub expiration_date: u64,
}

impl ZeroCopySerialize for RotationState {
    fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
        ByteWriter::new(data)
            .write_pubkey(&self.owner)?
            .write_bytes(&option_as_bytes(&self.new_owner, pubkey_as_bytes))?
            .write_u64(self.expiration_date)?
            .complete()
    }
}

impl ZeroCopyDeserialize for RotationState {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_pubkey(|x| &mut x.owner)?
            .read_option(|x| &mut x.new_owner, to_pubkey)?
            .read_u64(|x| &mut x.expiration_date)?
            .complete()
    }
}

impl Space for RotationState {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// get by user: Pubkey
#[derive(Default, Debug, PartialEq)]
pub struct UserId {
    pub id: u32,
    pub is_open: bool,
    pub is_activated: bool,
    pub account_bump: u8,
    pub rotation_state_bump: u8,
}

impl ZeroCopySerialize for UserId {
    fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
        ByteWriter::new(data)
            .write_u32(self.id)?
            .write_bool(self.is_open)?
            .write_bool(self.is_activated)?
            .write_u8(self.account_bump)?
            .write_u8(self.rotation_state_bump)?
            .complete()
    }
}

impl ZeroCopyDeserialize for UserId {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_u32(|x| &mut x.id)?
            .read_bool(|x| &mut x.is_open)?
            .read_bool(|x| &mut x.is_activated)?
            .read_u8(|x| &mut x.account_bump)?
            .read_u8(|x| &mut x.rotation_state_bump)?
            .complete()
    }
}

impl Space for UserId {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// get by user_id: u32
#[derive(Default, Debug, PartialEq)]
pub struct UserAccount {
    /// encrypted user data
    pub data: String,
    /// encryption nonce
    pub nonce: u64,
    /// allocated storage capacity
    pub max_size: u32,
}

impl ZeroCopySerialize for UserAccount {
    fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
        ByteWriter::new(data)
            .write_string(&self.data)?
            .write_u64(self.nonce)?
            .write_u32(self.max_size)?
            .complete()
    }
}

impl ZeroCopyDeserialize for UserAccount {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_string(|x| &mut x.data)?
            .read_u64(|x| &mut x.nonce)?
            .read_u32(|x| &mut x.max_size)?
            .complete()
    }
}

impl UserAccount {
    pub fn get_space(max_size: u32) -> u64 {
        // String: 4 bytes length + content
        let data: usize = 4 + max_size as usize;
        // u64
        const NONCE: usize = 8;
        // u32
        const MAX_SIZE: usize = 4;

        (data + NONCE + MAX_SIZE) as u64
    }
}
