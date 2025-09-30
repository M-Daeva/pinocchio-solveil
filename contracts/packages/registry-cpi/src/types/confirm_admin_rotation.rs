use {
    crate::state::Discriminator,
    base::{traits::DataLen, types::Result},
    bytemuck::{Pod, Zeroable},
    codama::CodamaInstruction,
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
};

pub use crate::types::update_config::Accounts;

#[derive(CodamaInstruction)]
// #[codama(name = "ConfirmAdminRotation")]
#[test_ser(Discriminator::ConfirmAdminRotation)]
#[p_serde]
pub struct InstructionData {}
