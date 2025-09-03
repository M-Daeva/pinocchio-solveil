use {
    crate::converters::{deserialize, deserialize_mut},
    bytemuck::{Pod, Zeroable},
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

    pub fn update<F>(&mut self, f: F) -> ProgramResult
    where
        F: FnOnce(&mut T),
        T: Pod + Zeroable,
    {
        let data = self.load_mut()?;
        f(data);
        Ok(())
    }
}

/// for tests
#[cfg(feature = "dev")]
pub trait InstructionSerialize {
    fn serialize(&self) -> Result<Vec<u8>>;
}
