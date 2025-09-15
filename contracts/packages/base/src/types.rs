use {
    crate::{
        converters::{deserialize_mut_unchecked, deserialize_unchecked},
        helpers::{get_flag, set_flag},
        traits::DataLen,
    },
    bytemuck::{Pod, Zeroable},
    codama::CodamaType,
    macro_p_serde::p_serde,
    pinocchio::{
        account_info::{AccountInfo, Ref, RefMut},
        program_error::ProgramError,
        ProgramResult,
    },
    std::marker::PhantomData,
};

pub type Result<T> = std::result::Result<T, ProgramError>;

// Account data wrapper for immutable operations
pub struct StorageR<'a, T> {
    data: Ref<'a, [u8]>,
    _phantom: PhantomData<T>,
}

impl<'a, T> StorageR<'a, T>
where
    T: Pod + Zeroable,
{
    #[inline]
    pub fn load(account: &'a AccountInfo) -> Result<Self> {
        Ok(Self {
            data: account.try_borrow_data()?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T> core::ops::Deref for StorageR<'a, T>
where
    T: Pod + Zeroable,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: We control the data format and ensure it's valid
        unsafe { deserialize_unchecked(&self.data) }
    }
}

// Account data wrapper for mutable operations
pub struct StorageW<'a, T> {
    data: RefMut<'a, [u8]>,
    _phantom: PhantomData<T>,
}

impl<'a, T> StorageW<'a, T>
where
    T: Pod + Zeroable,
{
    #[inline]
    pub fn load(account: &'a AccountInfo) -> Result<Self> {
        Ok(Self {
            data: account.try_borrow_mut_data()?,
            _phantom: PhantomData,
        })
    }

    #[inline]
    pub fn update<F>(account: &'a AccountInfo, f: F) -> ProgramResult
    where
        F: FnOnce(&mut T) -> ProgramResult,
    {
        f(&mut *Self::load(account)?)
    }
}

impl<'a, T> core::ops::Deref for StorageW<'a, T>
where
    T: Pod + Zeroable,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: We control the data format and ensure it's valid
        unsafe { deserialize_unchecked(&self.data) }
    }
}

impl<'a, T> core::ops::DerefMut for StorageW<'a, T>
where
    T: Pod + Zeroable,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: We control the data format and ensure it's valid
        unsafe { deserialize_mut_unchecked(&mut self.data) }
    }
}

#[derive(CodamaType)]
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

#[derive(CodamaType)]
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

#[derive(CodamaType)]
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

#[derive(CodamaType)]
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

#[derive(CodamaType)]
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
