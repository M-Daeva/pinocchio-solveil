use base::{
    converters::{to_u8, ByteWriter},
    guards::check_ix_data_len,
    helpers::get_space,
    types::{Result, Space, ZeroCopyDeserialize, ZeroCopySerialize},
};

pub mod discriminator {
    pub const INIT: u8 = 0;
    pub const SET: u8 = 1;
}

pub mod seed {
    pub const COUNTER: &[u8] = b"counter";
}

#[derive(Debug, PartialEq)]
pub struct Counter {
    pub value: u8,
}

impl ZeroCopySerialize for Counter {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        let mut writer = ByteWriter::new(data);
        writer.write_u8(self.value)?;

        Ok(())
    }
}

impl ZeroCopyDeserialize for Counter {
    fn deserialize_from(data: &[u8]) -> Result<Self> {
        let (value, end_index) = to_u8(data, 0)?;
        check_ix_data_len(data, end_index)?;

        Ok(Self { value })
    }
}

impl Space for Counter {
    fn get_space() -> u64 {
        get_space::<Self>()
    }
}
