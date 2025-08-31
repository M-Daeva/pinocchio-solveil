use {
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
    fn get_space() -> u64;
}

// Zero-copy serialization trait
pub trait ZeroCopySerialize {
    fn serialize_into(&self, data: &mut [u8]) -> ProgramResult;
}

pub trait ZeroCopyDeserialize: Sized {
    /// returns end_index
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)>;
}

// Account data wrapper for typed access
pub struct AccountData<'a, T> {
    data: RefMut<'a, [u8]>,
    _phantom: PhantomData<T>,
}

impl<'a, T> AccountData<'a, T>
where
    T: ZeroCopySerialize + ZeroCopyDeserialize,
{
    #[inline]
    pub fn init(account: &'a AccountInfo) -> Result<Self> {
        Ok(Self {
            data: account.try_borrow_mut_data()?,
            _phantom: PhantomData,
        })
    }

    #[inline]
    pub fn load(&self) -> Result<T> {
        T::deserialize_from(&self.data, 0).map(|(x, _)| x)
    }

    #[inline]
    pub fn save(&mut self, value: T) -> ProgramResult {
        value.serialize_into(&mut self.data)
    }

    #[inline]
    pub fn update<F>(&mut self, f: F) -> ProgramResult
    where
        F: FnOnce(T) -> Result<T>,
    {
        let data = self.load()?;
        let updated_data = f(data)?;
        self.save(updated_data)
    }
}

/// for tests
#[cfg(feature = "dev")]
pub trait InstructionSerialize {
    fn serialize(&self) -> Result<Vec<u8>>;
}
