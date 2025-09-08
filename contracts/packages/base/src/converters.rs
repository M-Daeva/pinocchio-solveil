use {
    crate::types::Result,
    bytemuck::{
        bytes_of, bytes_of_mut,
        checked::{try_from_bytes, try_from_bytes_mut},
        Pod, Zeroable,
    },
    pinocchio::program_error::ProgramError,
};

#[inline]
pub fn serialize<T>(data: &T) -> &[u8]
where
    T: Pod + Zeroable,
{
    bytes_of(data)
}

#[inline]
pub fn serialize_mut<T>(data: &mut T) -> &mut [u8]
where
    T: Pod + Zeroable,
{
    bytes_of_mut(data)
}

#[inline]
pub fn deserialize<T>(data: &[u8]) -> Result<&T>
where
    T: Pod + Zeroable,
{
    try_from_bytes(data).map_err(|_| ProgramError::InvalidAccountData)
}

#[inline]
pub unsafe fn deserialize_unchecked<T: Pod>(data: &[u8]) -> &T {
    &*(data.as_ptr() as *const T)
}

#[inline]
pub fn deserialize_mut<T>(data: &mut [u8]) -> Result<&mut T>
where
    T: Pod + Zeroable,
{
    try_from_bytes_mut(data).map_err(|_| ProgramError::InvalidAccountData)
}

#[inline]
pub unsafe fn deserialize_mut_unchecked<T: Pod>(data: &mut [u8]) -> &mut T {
    &mut *(data.as_mut_ptr() as *mut T)
}

// u64 (8 bytes, little-endian)
#[inline]
pub fn to_u64(data: &[u8], start_index: usize) -> Result<(u64, usize)> {
    const DATA_SIZE: usize = 8;
    let end_index = start_index + DATA_SIZE;
    let data_slice = &data[start_index..end_index];

    let value = u64::from_le_bytes(
        data_slice
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    Ok((value, end_index))
}
