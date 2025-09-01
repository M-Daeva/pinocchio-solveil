use {
    crate::state::discriminator as DISCRIMINATOR,
    base::{
        converters::ByteReader,
        types::{Result, ZeroCopyDeserialize},
    },
    macro_test_ser::test_serialize,
    macro_zc_serde::ZCDeserialize,
};

pub use crate::types::update_config::Accounts;

#[repr(C)]
#[derive(Default, ZCDeserialize)]
#[test_serialize(DISCRIMINATOR::CONFIRM_ADMIN_ROTATION)]
pub struct InstructionData {}

// impl TryFrom<&[u8]> for InstructionData {
//     type Error = ProgramError;

//     fn try_from(data: &[u8]) -> Result<Self> {
//         ByteReader::new::<Self>(data, 0).complete().map(|(x, _)| x)
//     }
// }

// /// for tests
// #[cfg(feature = "dev")]
// impl base::types::InstructionSerialize for InstructionData {
//     fn serialize(&self) -> Result<Vec<u8>> {
//         use base::converters::{ByteWriter, ByteWriterVecExt};

//         let mut buffer = vec![];
//         let position = ByteWriter::from_vec(&mut buffer)
//             .write_u8(DISCRIMINATOR::CONFIRM_ADMIN_ROTATION)?
//             .position();
//         buffer.truncate(position);

//         Ok(buffer)
//     }
// }
