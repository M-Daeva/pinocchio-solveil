use {
    crate::types::common::{AssetItem, Range},
    base::{
        converters::{
            option_as_bytes, pubkey_as_bytes, to_bool, to_option, to_pubkey, to_u32, to_u64, to_u8,
            ByteWriter,
        },
        guards::check_ix_data_len,
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
        let mut writer = ByteWriter::new(data);
        writer.write_u8(self.config)?;
        writer.write_u8(self.user_counter)?;
        writer.write_u8(self.rotation_state)?;

        Ok(())
    }
}

impl ZeroCopyDeserialize for Bump {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let (config, end_index) = to_u8(data, start_index)?;
        let (user_counter, end_index) = to_u8(data, end_index)?;
        let (rotation_state, end_index) = to_u8(data, end_index)?;
        check_ix_data_len(data, end_index)?;

        Ok((
            Self {
                config,
                user_counter,
                rotation_state,
            },
            end_index,
        ))
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
        let mut writer = ByteWriter::new(data);
        writer.write_pubkey(&self.admin)?;
        writer.write_bool(self.is_paused)?;
        writer.write_u32(self.rotation_timeout)?;
        writer.write_custom::<AssetItem>(&self.registration_fee)?;
        writer.write_custom::<Range>(&self.data_size_range)?;

        Ok(())
    }
}

impl ZeroCopyDeserialize for Config {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let (admin, end_index) = to_pubkey(data, start_index)?;
        let (is_paused, end_index) = to_bool(data, end_index)?;
        let (rotation_timeout, end_index) = to_u32(data, end_index)?;
        let (registration_fee, end_index) = AssetItem::deserialize_from(data, end_index)?;
        let (data_size_range, end_index) = Range::deserialize_from(data, end_index)?;
        check_ix_data_len(data, end_index)?;

        Ok((
            Self {
                admin,
                is_paused,
                rotation_timeout,
                registration_fee,
                data_size_range,
            },
            end_index,
        ))
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
        let mut writer = ByteWriter::new(data);
        writer.write_u32(self.last_user_id)?;

        Ok(())
    }
}

impl ZeroCopyDeserialize for UserCounter {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let (last_user_id, end_index) = to_u32(data, start_index)?;
        check_ix_data_len(data, end_index)?;

        Ok((Self { last_user_id }, end_index))
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
        let mut writer = ByteWriter::new(data);
        writer.write_pubkey(&self.owner)?;
        writer.write_bytes(&option_as_bytes(&self.new_owner, pubkey_as_bytes))?;
        writer.write_u64(self.expiration_date)?;

        Ok(())
    }
}

impl ZeroCopyDeserialize for RotationState {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let (owner, end_index) = to_pubkey(data, start_index)?;
        let (new_owner, end_index) = to_option(data, end_index, to_pubkey)?;
        let (expiration_date, end_index) = to_u64(data, end_index)?;
        check_ix_data_len(data, end_index)?;

        Ok((
            Self {
                owner,
                new_owner,
                expiration_date,
            },
            end_index,
        ))
    }
}

impl Space for RotationState {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}

// /// get by user: Pubkey
// #[account]
// #[derive(InitSpace, PartialEq, Debug)]
// pub struct UserId {
//     pub id: u32,
//     pub is_open: bool,
//     pub is_activated: bool,
//     pub account_bump: u8,
//     pub rotation_state_bump: u8,
// }

// /// get by user_id: u32
// #[account]
// #[derive(PartialEq, Debug)]
// pub struct UserAccount {
//     /// encrypted user data
//     pub data: String,
//     /// encryption nonce
//     pub nonce: u64,
//     /// allocated storage capacity
//     pub max_size: u32,
// }

// impl UserAccount {
//     pub fn get_space(max_size: u32) -> usize {
//         8 +   // discriminator
//         4 + max_size as usize + // data (String: 4 bytes length + content)
//         8 +   // nonce (u64)
//         4 // max_size (u32)
//     }
// }
