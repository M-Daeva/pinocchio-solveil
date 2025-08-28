use {
    base::{
        converters::{to_pubkey, to_u32, to_u64, ByteWriter},
        guards::check_ix_data_len,
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
        let (amount, end_index) = to_u64(data, start_index)?;
        let (asset, end_index) = to_pubkey(data, end_index)?;
        check_ix_data_len(data, end_index)?;

        Ok((Self { amount, asset }, end_index))
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
        let (min, end_index) = to_u32(data, start_index)?;
        let (max, end_index) = to_u32(data, end_index)?;
        check_ix_data_len(data, end_index)?;

        Ok((Self { min, max }, end_index))
    }
}
