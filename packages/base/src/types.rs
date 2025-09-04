use {
    crate::{
        converters::{deserialize, deserialize_mut},
        helpers::{get_flag, set_flag},
    },
    bytemuck::{Pod, Zeroable},
    macro_zc_serde::p_serde,
    pinocchio::{
        account_info::{AccountInfo, RefMut},
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_pubkey::pubkey,
    std::marker::PhantomData,
};

pub const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
pub const TOKEN_2022_MINT_DISCRIMINATOR: u8 = 0x01;
pub const TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub type Result<T> = std::result::Result<T, ProgramError>;

pub trait ErrorIndexOffset {
    const OFFSET: u32;
}

pub trait Space {
    fn get_space() -> usize;
}

// Account data wrapper for typed access
pub struct Storage<'a, T> {
    data: RefMut<'a, [u8]>,
    _phantom: PhantomData<T>,
}

impl<'a, T> Storage<'a, T>
where
    T: Pod + Zeroable,
{
    #[inline]
    pub fn init(account: &'a AccountInfo) -> Result<Self> {
        Ok(Self {
            data: account.try_borrow_mut_data()?,
            _phantom: PhantomData,
        })
    }

    #[inline]
    pub fn load(&self) -> Result<&T> {
        deserialize(&self.data)
    }

    #[inline]
    pub fn load_mut(&mut self) -> Result<&mut T> {
        deserialize_mut(&mut self.data)
    }

    #[inline]
    pub fn update<F>(&mut self, f: F) -> ProgramResult
    where
        F: FnOnce(&mut T) -> ProgramResult,
        T: Pod + Zeroable,
    {
        let data = self.load_mut()?;
        f(data)
    }
}

#[p_serde]
pub struct BitField(u8);

impl From<bool> for BitField {
    #[inline]
    fn from(x: bool) -> Self {
        if x {
            Self(1)
        } else {
            Self(0)
        }
    }
}

impl BitField {
    #[inline]
    pub fn get_raw(&self) -> u8 {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: u8) {
        self.0 = x;
    }

    #[inline]
    pub fn get_flag(&self, bit: u8) -> bool {
        get_flag(self.0, bit)
    }

    #[inline]
    pub fn set_flag(&mut self, bit: u8, value: bool) {
        self.0 = set_flag(self.0, bit, value);
    }

    #[inline]
    pub fn get_bit(&self) -> bool {
        get_flag(self.0, 0)
    }

    #[inline]
    pub fn set_bit(&mut self, value: bool) {
        self.0 = set_flag(self.0, 0, value);
    }
}

#[p_serde]
pub struct Uint16([u8; 2]);

impl From<u16> for Uint16 {
    #[inline]
    fn from(x: u16) -> Self {
        Self(x.to_le_bytes())
    }
}

impl Uint16 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 2] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 2]) {
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> u16 {
        u16::from_le_bytes(self.0)
    }

    #[inline]
    pub fn set(&mut self, x: u16) {
        self.0 = x.to_le_bytes();
    }
}

#[p_serde]
pub struct Uint32([u8; 4]);

impl From<u32> for Uint32 {
    #[inline]
    fn from(x: u32) -> Self {
        Self(x.to_le_bytes())
    }
}

impl Uint32 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 4] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 4]) {
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> u32 {
        u32::from_le_bytes(self.0)
    }

    #[inline]
    pub fn set(&mut self, x: u32) {
        self.0 = x.to_le_bytes();
    }
}

#[p_serde]
pub struct Uint64([u8; 8]);

impl From<u64> for Uint64 {
    #[inline]
    fn from(x: u64) -> Self {
        Self(x.to_le_bytes())
    }
}

impl Uint64 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 8] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 8]) {
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> u64 {
        u64::from_le_bytes(self.0)
    }

    #[inline]
    pub fn set(&mut self, x: u64) {
        self.0 = x.to_le_bytes();
    }
}

#[p_serde]
pub struct Uint128([u8; 16]);

impl From<u128> for Uint128 {
    #[inline]
    fn from(x: u128) -> Self {
        Self(x.to_le_bytes())
    }
}

impl Uint128 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 16] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 16]) {
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> u128 {
        u128::from_le_bytes(self.0)
    }

    #[inline]
    pub fn set(&mut self, x: u128) {
        self.0 = x.to_le_bytes();
    }
}

/// for tests
#[cfg(feature = "dev")]
pub trait InstructionSerialize {
    fn serialize(&self) -> Result<Vec<u8>>;
}
