use {
    base::{
        converters::{ByteReader, ByteWriter},
        types::{Result, ZeroCopyDeserialize, ZeroCopySerialize},
    },
    pinocchio::pubkey::Pubkey,
};

#[derive(Debug, PartialEq)]
pub struct AssetItem {
    pub amount: u64,
    pub asset: Pubkey,
}

impl ZeroCopySerialize for AssetItem {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        ByteWriter::new(data)
            .write_u64(self.amount)?
            .write_pubkey(&self.asset)?
            .complete()
    }
}

impl ZeroCopyDeserialize for AssetItem {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            amount: b.read_u64()?,
            asset: b.read_pubkey()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}

#[derive(Debug, PartialEq)]
pub struct Range {
    pub min: u32,
    pub max: u32,
}

impl ZeroCopySerialize for Range {
    fn serialize_into(&self, data: &mut [u8]) -> Result<()> {
        ByteWriter::new(data)
            .write_u32(self.min)?
            .write_u32(self.max)?
            .complete()
    }
}

impl ZeroCopyDeserialize for Range {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        let mut b = ByteReader::new(data, start_index);

        let data = Self {
            min: b.read_u32()?,
            max: b.read_u32()?,
        };

        Ok((data, b.check_and_get_end_index()?))
    }
}
