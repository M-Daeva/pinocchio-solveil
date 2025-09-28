use {
    crate::{
        converters::{deserialize_mut_unchecked, deserialize_unchecked},
        helpers::{get_flag, set_flag},
        traits::DataLen,
    },
    bytemuck::{Pod, Zeroable},
    codama::CodamaType,
    core::marker::PhantomData,
    macro_p_serde::p_serde,
    pinocchio::{
        account_info::{AccountInfo, Ref, RefMut},
        program_error::ProgramError,
        ProgramResult,
    },
    pinocchio_ts_generator::GenerateTS,
};

pub type Result<T> = core::result::Result<T, ProgramError>;

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

#[derive(CodamaType, GenerateTS)]
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

#[derive(CodamaType, GenerateTS)]
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

#[derive(CodamaType, GenerateTS)]
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

#[derive(CodamaType, GenerateTS)]
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

#[derive(CodamaType, GenerateTS)]
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

#[derive(CodamaType, GenerateTS)]
#[p_serde]
pub struct String16([u8; 16]);

impl From<&str> for String16 {
    fn from(x: &str) -> Self {
        let len = x.len();
        assert!(len < 16, "String too long for String16 (last byte is NUL)");
        let mut buf = [0u8; 16];
        buf[..len].copy_from_slice(x.as_bytes());
        Self(buf)
    }
}

impl String16 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 16] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 16]) {
        assert!(
            x[15] == 0 || x.iter().any(|&y| y == 0),
            "Raw buffer must contain a NUL terminator"
        );
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> &str {
        let end = self.0.iter().position(|&x| x == 0).unwrap_or(self.0.len());
        core::str::from_utf8(&self.0[..end]).unwrap()
    }

    #[inline]
    pub fn set(&mut self, x: &str) {
        let len = x.len();
        assert!(len < 16, "String too long for String16 (last byte is NUL)");
        self.0[..len].copy_from_slice(x.as_bytes());
        self.0[len] = 0;
    }
}

#[derive(CodamaType, GenerateTS)]
#[p_serde]
pub struct String32([u8; 32]);

impl From<&str> for String32 {
    fn from(x: &str) -> Self {
        let len = x.len();
        assert!(len < 32, "String too long for String32 (last byte is NUL)");
        let mut buf = [0u8; 32];
        buf[..len].copy_from_slice(x.as_bytes());
        Self(buf)
    }
}

impl String32 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 32] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 32]) {
        assert!(
            x[31] == 0 || x.iter().any(|&y| y == 0),
            "Raw buffer must contain a NUL terminator"
        );
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> &str {
        let end = self.0.iter().position(|&x| x == 0).unwrap_or(self.0.len());
        core::str::from_utf8(&self.0[..end]).unwrap()
    }

    #[inline]
    pub fn set(&mut self, x: &str) {
        let len = x.len();
        assert!(len < 32, "String too long for String32 (last byte is NUL)");
        self.0[..len].copy_from_slice(x.as_bytes());
        self.0[len] = 0;
    }
}

#[derive(CodamaType, GenerateTS, Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct String64([u8; 64]);

impl Default for String64 {
    #[inline]
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl DataLen for String64 {
    const LEN: usize = core::mem::size_of::<Self>();
}

impl From<&str> for String64 {
    fn from(x: &str) -> Self {
        let len = x.len();
        assert!(len < 64, "String too long for String64 (last byte is NUL)");
        let mut buf = [0u8; 64];
        buf[..len].copy_from_slice(x.as_bytes());
        Self(buf)
    }
}

impl String64 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 64] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 64]) {
        assert!(
            x[63] == 0 || x.iter().any(|&y| y == 0),
            "Raw buffer must contain a NUL terminator"
        );
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> &str {
        let end = self.0.iter().position(|&x| x == 0).unwrap_or(self.0.len());
        core::str::from_utf8(&self.0[..end]).unwrap()
    }

    #[inline]
    pub fn set(&mut self, x: &str) {
        let len = x.len();
        assert!(len < 64, "String too long for String64 (last byte is NUL)");
        self.0[..len].copy_from_slice(x.as_bytes());
        self.0[len] = 0;
    }
}

#[derive(CodamaType, GenerateTS, Debug, PartialEq, Eq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct String4096([u8; 4096]);

impl Default for String4096 {
    #[inline]
    fn default() -> Self {
        Self([0u8; 4096])
    }
}

impl DataLen for String4096 {
    const LEN: usize = core::mem::size_of::<Self>();
}

impl From<&str> for String4096 {
    fn from(x: &str) -> Self {
        let len = x.len();
        assert!(
            len < 4096,
            "String too long for String4096 (last byte is NUL)"
        );
        let mut buf = [0u8; 4096];
        buf[..len].copy_from_slice(x.as_bytes());
        Self(buf)
    }
}

impl String4096 {
    #[inline]
    pub fn get_raw(&self) -> [u8; 4096] {
        self.0
    }

    #[inline]
    pub fn set_raw(&mut self, x: [u8; 4096]) {
        assert!(
            x[4095] == 0 || x.iter().any(|&y| y == 0),
            "Raw buffer must contain a NUL terminator"
        );
        self.0 = x;
    }

    #[inline]
    pub fn get(&self) -> &str {
        let end = self.0.iter().position(|&x| x == 0).unwrap_or(self.0.len());
        core::str::from_utf8(&self.0[..end]).unwrap()
    }

    #[inline]
    pub fn set(&mut self, x: &str) {
        let len = x.len();
        assert!(
            len < 4096,
            "String too long for String4096 (last byte is NUL)"
        );
        self.0[..len].copy_from_slice(x.as_bytes());
        self.0[len] = 0;
    }
}
