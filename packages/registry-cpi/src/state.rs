use {
    crate::types::common::{AssetItem, Range},
    base::{
        converters::{option_as_bytes, pubkey_as_bytes, to_pubkey, ByteReader, ByteWriter},
        helpers::get_space,
        types::{Result, Space, ZeroCopyDeserialize, ZeroCopySerialize},
    },
    pinocchio::pubkey::Pubkey,
    pinocchio_pubkey::pubkey,
};

pub mod discriminator {
    pub const INIT: u8 = 0;
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
#[derive(Debug, PartialEq)]
pub struct Bump {
    pub config: u8,
    pub user_counter: u8,
    pub rotation_state: u8,
}

impl ZeroCopySerialize for Bump {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        ByteWriter::new(data)
            .write_u8(self.config)?
            .write_u8(self.user_counter)?
            .write_u8(self.rotation_state)?
            .complete()
    }
}

impl ZeroCopyDeserialize for Bump {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            config: b.read_u8()?,
            user_counter: b.read_u8()?,
            rotation_state: b.read_u8()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}

impl Space for Bump {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

#[derive(Debug, PartialEq)]
pub struct Config {
    /// can update the config and execute priveledged instructions
    pub admin: Pubkey,
    pub is_paused: bool,
    pub rotation_timeout: u32,
    pub registration_fee: AssetItem,
    pub data_size_range: Range,
}

impl ZeroCopySerialize for Config {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        ByteWriter::new(data)
            .write_pubkey(&self.admin)?
            .write_bool(self.is_paused)?
            .write_u32(self.rotation_timeout)?
            .write_custom::<AssetItem>(&self.registration_fee)?
            .write_custom::<Range>(&self.data_size_range)?
            .complete()
    }
}

impl ZeroCopyDeserialize for Config {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            admin: b.read_pubkey()?,
            is_paused: b.read_bool()?,
            rotation_timeout: b.read_u32()?,
            registration_fee: b.read::<AssetItem>()?,
            data_size_range: b.read::<Range>()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}

impl Space for Config {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// for indexing
#[derive(Debug, PartialEq, Default)]
pub struct UserCounter {
    pub last_user_id: u32,
}

impl ZeroCopySerialize for UserCounter {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        ByteWriter::new(data)
            .write_u32(self.last_user_id)?
            .complete()
    }
}

impl ZeroCopyDeserialize for UserCounter {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            last_user_id: b.read_u32()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}

impl Space for UserCounter {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// to transfer ownership from one address to another in 2 steps (for security reasons) \
/// used both for app admin and user accounts
#[derive(Debug, PartialEq)]
pub struct RotationState {
    pub owner: Pubkey,
    pub new_owner: Option<Pubkey>,
    pub expiration_date: u64,
}

impl ZeroCopySerialize for RotationState {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        ByteWriter::new(data)
            .write_pubkey(&self.owner)?
            .write_bytes(&option_as_bytes(&self.new_owner, pubkey_as_bytes))?
            .write_u64(self.expiration_date)?
            .complete()
    }
}

impl ZeroCopyDeserialize for RotationState {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            owner: b.read_pubkey()?,
            new_owner: b.read_option(to_pubkey)?,
            expiration_date: b.read_u64()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}

impl Space for RotationState {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// get by user: Pubkey
#[derive(Debug, PartialEq)]
pub struct UserId {
    pub id: u32,
    pub is_open: bool,
    pub is_activated: bool,
    pub account_bump: u8,
    pub rotation_state_bump: u8,
}

impl ZeroCopySerialize for UserId {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
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
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            id: b.read_u32()?,
            is_open: b.read_bool()?,
            is_activated: b.read_bool()?,
            account_bump: b.read_u8()?,
            rotation_state_bump: b.read_u8()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}

impl Space for UserId {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

/// get by user_id: u32
#[derive(Debug, PartialEq)]
pub struct UserAccount {
    /// encrypted user data
    pub data: String,
    /// encryption nonce
    pub nonce: u64,
    /// allocated storage capacity
    pub max_size: u32,
}

impl ZeroCopySerialize for UserAccount {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        ByteWriter::new(data)
            .write_string(&self.data)?
            .write_u64(self.nonce)?
            .write_u32(self.max_size)?
            .complete()
    }
}

impl ZeroCopyDeserialize for UserAccount {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            data: b.read_string()?,
            nonce: b.read_u64()?,
            max_size: b.read_u32()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}

impl UserAccount {
    pub fn get_space(max_size: u32) -> usize {
        8 +   // discriminator
        4 + max_size as usize + // data (String: 4 bytes length + content)
        8 +   // nonce (u64)
        4 // max_size (u32)
    }
}
