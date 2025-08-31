use {
    crate::{
        guards::check_ix_data_len,
        types::{Result, ZeroCopyDeserialize, ZeroCopySerialize},
    },
    pinocchio::{program_error::ProgramError, pubkey::Pubkey, ProgramResult},
    std::{mem, slice},
};

#[repr(C)]
pub struct ByteReader<'a, T> {
    data: &'a [u8],
    position: usize,
    value: T,
}

impl<'a> ByteReader<'a, ()> {
    #[inline]
    pub fn new<T: Default>(data: &'a [u8], start_index: usize) -> ByteReader<'a, T> {
        ByteReader {
            data,
            position: start_index,
            value: T::default(),
        }
    }
}

impl<'a, T> ByteReader<'a, T> {
    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }

    #[inline]
    pub fn complete(self) -> Result<(T, usize)> {
        check_ix_data_len(self.data, self.position)?;
        Ok((self.value, self.position))
    }

    // Generic field updater - the core of the fluent interface
    #[inline]
    fn update_field<F, V>(
        mut self,
        field_getter: F,
        parser: fn(&[u8], usize) -> Result<(V, usize)>,
    ) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut V,
    {
        let (value, new_pos) = parser(self.data, self.position)?;
        *field_getter(&mut self.value) = value;
        self.position = new_pos;
        Ok(self)
    }

    // Convenience methods for common types
    #[inline]
    pub fn read_bool<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut bool,
    {
        self.update_field(field_getter, to_bool)
    }

    #[inline]
    pub fn read_u8<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut u8,
    {
        self.update_field(field_getter, to_u8)
    }

    #[inline]
    pub fn read_u16<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut u16,
    {
        self.update_field(field_getter, to_u16)
    }

    #[inline]
    pub fn read_u32<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut u32,
    {
        self.update_field(field_getter, to_u32)
    }

    #[inline]
    pub fn read_u64<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut u64,
    {
        self.update_field(field_getter, to_u64)
    }

    #[inline]
    pub fn read_u128<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut u128,
    {
        self.update_field(field_getter, to_u128)
    }

    #[inline]
    pub fn read_pubkey<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut Pubkey,
    {
        self.update_field(field_getter, to_pubkey)
    }

    #[inline]
    pub fn read_string<F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut String,
    {
        self.update_field(field_getter, to_string)
    }

    // For Vec<T> with fixed-size elements
    #[inline]
    pub fn read_vec_fixed<V, F, P>(self, field_getter: F, item_parser: P) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut Vec<V>,
        P: Fn(&[u8], usize) -> Result<(V, usize)>,
    {
        let (value, new_pos) = to_vec_fixed(self.data, self.position, item_parser)?;
        let mut updated_self = self;
        *field_getter(&mut updated_self.value) = value;
        updated_self.position = new_pos;
        Ok(updated_self)
    }

    // For Option<T>
    #[inline]
    pub fn read_option<V, F, P>(self, field_getter: F, item_parser: P) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut Option<V>,
        P: Fn(&[u8], usize) -> Result<(V, usize)>,
    {
        let (value, new_pos) = to_option(self.data, self.position, item_parser)?;
        let mut updated_self = self;
        *field_getter(&mut updated_self.value) = value;
        updated_self.position = new_pos;
        Ok(updated_self)
    }

    // For arrays [T; N]
    #[inline]
    pub fn read_array<V, F, P, const N: usize>(
        self,
        field_getter: F,
        item_parser: P,
    ) -> Result<Self>
    where
        V: Copy + Default,
        F: FnOnce(&mut T) -> &mut [V; N],
        P: Fn(&[u8], usize) -> Result<(V, usize)>,
    {
        let (value, new_pos) = to_array(self.data, self.position, item_parser)?;
        let mut updated_self = self;
        *field_getter(&mut updated_self.value) = value;
        updated_self.position = new_pos;
        Ok(updated_self)
    }

    // For complex types with custom parsers
    #[inline]
    pub fn read_with_parser<V, F, P>(self, field_getter: F, parser: P) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut V,
        P: Fn(&[u8], usize) -> Result<(V, usize)>,
    {
        let (value, new_pos) = parser(self.data, self.position)?;
        let mut updated_self = self;
        *field_getter(&mut updated_self.value) = value;
        updated_self.position = new_pos;
        Ok(updated_self)
    }

    // For types that implement ZeroCopyDeserialize
    #[inline]
    pub fn read_custom<V: ZeroCopyDeserialize, F>(self, field_getter: F) -> Result<Self>
    where
        F: FnOnce(&mut T) -> &mut V,
    {
        let (value, new_pos) = V::deserialize_from(self.data, self.position)?;
        let mut updated_self = self;
        *field_getter(&mut updated_self.value) = value;
        updated_self.position = new_pos;
        Ok(updated_self)
    }
}

// Boolean (1 byte: 0 = false, non-zero = true)
#[inline]
pub fn to_bool(data: &[u8], start_index: usize) -> Result<(bool, usize)> {
    const DATA_SIZE: usize = 1;
    let end_index = start_index + DATA_SIZE;
    let value = data[start_index] != 0;

    Ok((value, end_index))
}

// u8 (1 byte)
#[inline]
pub fn to_u8(data: &[u8], start_index: usize) -> Result<(u8, usize)> {
    const DATA_SIZE: usize = 1;
    let end_index = start_index + DATA_SIZE;
    let value = data[start_index];

    Ok((value, end_index))
}

// u16 (2 bytes, little-endian)
#[inline]
pub fn to_u16(data: &[u8], start_index: usize) -> Result<(u16, usize)> {
    const DATA_SIZE: usize = 2;
    let end_index = start_index + DATA_SIZE;
    let data_slice = &data[start_index..end_index];

    let value = u16::from_le_bytes(
        data_slice
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    Ok((value, end_index))
}

// u32 (4 bytes, little-endian)
#[inline]
pub fn to_u32(data: &[u8], start_index: usize) -> Result<(u32, usize)> {
    const DATA_SIZE: usize = 4;
    let end_index = start_index + DATA_SIZE;
    let data_slice = &data[start_index..end_index];

    let value = u32::from_le_bytes(
        data_slice
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    Ok((value, end_index))
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

// u128 (16 bytes, little-endian)
#[inline]
pub fn to_u128(data: &[u8], start_index: usize) -> Result<(u128, usize)> {
    const DATA_SIZE: usize = 16;
    let end_index = start_index + DATA_SIZE;
    let data_slice = &data[start_index..end_index];

    let value = u128::from_le_bytes(
        data_slice
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    Ok((value, end_index))
}

#[inline]
pub fn to_pubkey(data: &[u8], start_index: usize) -> Result<(Pubkey, usize)> {
    const DATA_SIZE: usize = 32;
    let end_index = start_index + DATA_SIZE;
    let data_slice = &data[start_index..end_index];

    let value = Pubkey::try_from(data_slice).map_err(|_| ProgramError::InvalidInstructionData)?;

    Ok((value, end_index))
}

// String (length-prefixed with u32, then UTF-8 bytes)
// Format: [length: u32][utf8_bytes: length bytes]
#[inline]
pub fn to_string(data: &[u8], start_index: usize) -> Result<(String, usize)> {
    const U32_DATA_SIZE: usize = 4;
    let (length, _) = to_u32(data, start_index)?;
    let content_start = start_index + U32_DATA_SIZE;
    let content_end = content_start + length as usize;

    let string = String::from_utf8(data[content_start..content_end].to_vec())
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    Ok((string, content_end))
}

// Vec<T> where T has a fixed size (like u64, u32, etc.)
// Format: [length: u32][items: length * size_of::<T>() bytes]
#[inline]
pub fn to_vec_fixed<T, F>(
    data: &[u8],
    start_index: usize,
    item_parser: F,
) -> Result<(Vec<T>, usize)>
where
    F: Fn(&[u8], usize) -> Result<(T, usize)>,
{
    let (length, mut current_index) = to_u32(data, start_index)?;
    let mut vec = Vec::with_capacity(length as usize);

    for _ in 0..length {
        let (item, end_index) = item_parser(data, current_index)?;
        vec.push(item);
        current_index = end_index;
    }

    Ok((vec, current_index))
}

// Option<T> (1 byte discriminant + optional T)
// Format: [is_some: u8][value: T if is_some != 0]
#[inline]
pub fn to_option<T, F>(
    data: &[u8],
    start_index: usize,
    item_parser: F,
) -> Result<(Option<T>, usize)>
where
    F: Fn(&[u8], usize) -> Result<(T, usize)>,
{
    let (is_some, next_index) = to_bool(data, start_index)?;

    if is_some {
        let (value, new_index) = item_parser(data, next_index)?;
        Ok((Some(value), new_index))
    } else {
        Ok((None, next_index))
    }
}

// Array of fixed-size elements [T; N]
// No length prefix - reads exactly N elements of type T
#[inline]
pub fn to_array<T, F, const N: usize>(
    data: &[u8],
    start_index: usize,
    item_parser: F,
) -> Result<([T; N], usize)>
where
    T: Copy + Default,
    F: Fn(&[u8], usize) -> Result<(T, usize)>,
{
    let mut array = [T::default(); N];
    let mut current_index = start_index;

    for i in 0..N {
        let (item, end_index) = item_parser(data, current_index)?;
        array[i] = item;
        current_index = end_index;
    }

    Ok((array, current_index))
}

#[repr(C)]
pub struct ByteWriter<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> ByteWriter<'a> {
    #[inline]
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    #[inline]
    pub fn write_custom<T>(mut self, data: &T) -> Result<Self>
    where
        T: ZeroCopySerialize + ZeroCopyDeserialize,
    {
        let end_pos = self.position + core::mem::size_of::<T>();
        if end_pos > self.buffer.len() {
            Err(ProgramError::InvalidInstructionData)?;
        }

        data.serialize_into(&mut self.buffer[self.position..])?;
        self.position = end_pos;

        Ok(self)
    }

    #[inline]
    pub fn write_option_custom<T>(mut self, value: &Option<T>) -> Result<Self>
    where
        T: ZeroCopySerialize + ZeroCopyDeserialize,
    {
        match value {
            Some(inner_value) => {
                self = self.write_u8(1)?; // is_some = true
                self = self.write_custom(inner_value)?;
            }
            None => {
                self = self.write_u8(0)?; // is_some = false
            }
        }
        Ok(self)
    }

    #[inline]
    pub fn write_option<T, F>(mut self, value: &Option<T>, converter: F) -> Result<Self>
    where
        F: FnOnce(&T) -> &[u8],
    {
        match value {
            Some(inner_value) => {
                self = self.write_u8(1)?; // is_some = true
                let bytes = converter(inner_value);
                self = self.write_bytes(bytes)?;
            }
            None => {
                self = self.write_u8(0)?; // is_some = false
            }
        }
        Ok(self)
    }

    #[inline]
    pub fn write_bytes(mut self, data: &[u8]) -> Result<Self> {
        let end_pos = self.position + data.len();
        if end_pos > self.buffer.len() {
            Err(ProgramError::InvalidInstructionData)?;
        }

        self.buffer[self.position..end_pos].copy_from_slice(data);
        self.position = end_pos;

        Ok(self)
    }

    #[inline]
    pub fn write_u8(self, value: u8) -> Result<Self> {
        self.write_bytes(&[value])
    }

    #[inline]
    pub fn write_u16(self, value: u16) -> Result<Self> {
        self.write_bytes(&value.to_le_bytes())
    }

    #[inline]
    pub fn write_u32(self, value: u32) -> Result<Self> {
        self.write_bytes(&value.to_le_bytes())
    }

    #[inline]
    pub fn write_u64(self, value: u64) -> Result<Self> {
        self.write_bytes(&value.to_le_bytes())
    }

    #[inline]
    pub fn write_u128(self, value: u128) -> Result<Self> {
        self.write_bytes(&value.to_le_bytes())
    }

    #[inline]
    pub fn write_bool(self, value: bool) -> Result<Self> {
        self.write_u8(if value { 1 } else { 0 })
    }

    #[inline]
    pub fn write_string(mut self, value: &str) -> Result<Self> {
        let bytes = value.as_bytes();
        self = self.write_u32(bytes.len() as u32)?;
        self.write_bytes(bytes)
    }

    #[inline]
    pub fn write_pubkey(self, value: &Pubkey) -> Result<Self> {
        self.write_bytes(value.as_ref())
    }

    #[inline]
    pub fn complete(self) -> ProgramResult {
        Ok(())
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.position]
    }
}

/// for tests
#[cfg(feature = "dev")]
pub trait ByteWriterVecExt<'a> {
    fn from_vec(buffer: &'a mut Vec<u8>) -> ByteWriter<'a>;
    fn truncate_buffer(self, buffer: &mut Vec<u8>);
}

/// for tests
#[cfg(feature = "dev")]
impl<'a> ByteWriterVecExt<'a> for ByteWriter<'a> {
    #[inline]
    fn from_vec(buffer: &'a mut Vec<u8>) -> ByteWriter<'a> {
        // Ensure the vec has some initial capacity to avoid immediate reallocation
        if buffer.capacity() < 256 {
            buffer.reserve(256);
        }
        // Resize to match capacity to avoid bounds checking issues
        let capacity = buffer.capacity();
        buffer.resize(capacity, 0);
        ByteWriter::new(buffer)
    }

    #[inline]
    fn truncate_buffer(self, buffer: &mut Vec<u8>) {
        buffer.truncate(self.position());
    }
}

// For types that can be directly interpreted as bytes (zero-copy)

// Boolean (1 byte: 0 = false, 1 = true)
#[inline]
pub fn bool_as_bytes(value: &bool) -> &[u8] {
    unsafe { slice::from_raw_parts(value as *const bool as *const u8, 1) }
}

// u8 (1 byte) - already a byte
#[inline]
pub fn u8_as_bytes(value: &u8) -> &[u8] {
    slice::from_ref(value)
}

// u16 (2 bytes, little-endian)
#[inline]
pub fn u16_as_bytes(value: &u16) -> &[u8] {
    unsafe { slice::from_raw_parts(value as *const u16 as *const u8, mem::size_of::<u16>()) }
}

// u32 (4 bytes, little-endian)
#[inline]
pub fn u32_as_bytes(value: &u32) -> &[u8] {
    unsafe { slice::from_raw_parts(value as *const u32 as *const u8, mem::size_of::<u32>()) }
}

// u64 (8 bytes, little-endian)
#[inline]
pub fn u64_as_bytes(value: &u64) -> &[u8] {
    unsafe { slice::from_raw_parts(value as *const u64 as *const u8, mem::size_of::<u64>()) }
}

// u128 (16 bytes, little-endian)
#[inline]
pub fn u128_as_bytes(value: &u128) -> &[u8] {
    unsafe { slice::from_raw_parts(value as *const u128 as *const u8, mem::size_of::<u128>()) }
}

// Pubkey (32 bytes) - assuming Pubkey has as_ref() or to_bytes() method
#[inline]
pub fn pubkey_as_bytes(value: &Pubkey) -> &[u8] {
    value.as_ref() // Most Pubkey implementations provide this
}

// String as UTF-8 bytes (no length prefix - just the content)
#[inline]
pub fn string_as_bytes(value: &str) -> &[u8] {
    value.as_bytes()
}

// Slices of primitive types (zero-copy)
#[inline]
pub fn slice_u16_as_bytes(value: &[u16]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            value.as_ptr() as *const u8,
            value.len() * mem::size_of::<u16>(),
        )
    }
}

#[inline]
pub fn slice_u32_as_bytes(value: &[u32]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            value.as_ptr() as *const u8,
            value.len() * mem::size_of::<u32>(),
        )
    }
}

#[inline]
pub fn slice_u64_as_bytes(value: &[u64]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            value.as_ptr() as *const u8,
            value.len() * mem::size_of::<u64>(),
        )
    }
}

#[inline]
pub fn slice_u128_as_bytes(value: &[u128]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            value.as_ptr() as *const u8,
            value.len() * mem::size_of::<u128>(),
        )
    }
}

// Option<T> (1 byte discriminant + optional T)
// Format: [is_some: u8][value: T if is_some != 0]
#[inline]
pub fn option_as_bytes<T, F>(value: &Option<T>, item_serializer: F) -> Vec<u8>
where
    F: Fn(&T) -> &[u8],
{
    match value {
        Some(inner_value) => {
            let mut bytes = Vec::new();
            bytes.push(1u8); // is_some = true
            bytes.extend_from_slice(item_serializer(inner_value));
            bytes
        }
        None => {
            vec![0u8] // is_some = false, no additional data
        }
    }
}
