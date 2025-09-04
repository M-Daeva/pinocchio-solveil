use crate::types::Result;

pub trait ErrorIndexOffset {
    const OFFSET: u32;
}

pub trait Space {
    fn get_space() -> usize;
}

/// for tests
#[cfg(feature = "dev")]
pub trait InstructionSerialize {
    fn serialize(&self) -> Result<Vec<u8>>;
}
