use {
    base::{
        converters::{ByteReader, ByteWriter},
        types::{Result, ZeroCopyDeserialize, ZeroCopySerialize},
    },
    macro_zc_serde::ZCSerialize,
    pinocchio::{pubkey::Pubkey, ProgramResult},
};

#[repr(C)]
#[derive(Default, Debug, PartialEq, ZCSerialize)]
pub struct AssetItem {
    pub amount: u64,
    pub asset: Pubkey,
}

// impl ZeroCopySerialize for AssetItem {
//     fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
//         ByteWriter::new(data)
//             .write_u64(self.amount)?
//             .write_pubkey(&self.asset)?
//             .complete()
//     }
// }

impl ZeroCopyDeserialize for AssetItem {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_u64(|x| &mut x.amount)?
            .read_pubkey(|x| &mut x.asset)?
            .complete()
    }
}

#[repr(C)]
#[derive(Default, Debug, PartialEq, ZCSerialize)]
pub struct Range {
    pub min: u32,
    pub max: u32,
}

// impl ZeroCopySerialize for Range {
//     fn serialize_into(&self, data: &mut [u8]) -> ProgramResult {
//         ByteWriter::new(data)
//             .write_u32(self.min)?
//             .write_u32(self.max)?
//             .complete()
//     }
// }

impl ZeroCopyDeserialize for Range {
    fn deserialize_from(data: &[u8], start_index: usize) -> Result<(Self, usize)> {
        ByteReader::new::<Self>(data, start_index)
            .read_u32(|x| &mut x.min)?
            .read_u32(|x| &mut x.max)?
            .complete()
    }
}
