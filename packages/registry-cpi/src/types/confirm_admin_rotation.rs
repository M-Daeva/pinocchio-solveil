use {
    crate::state::Discriminator,
    base::{traits::DataLen, types::Result},
    bytemuck::{Pod, Zeroable},
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
};

pub use crate::types::update_config::Accounts;

#[test_ser(Discriminator::ConfirmAdminRotation)]
#[p_serde]
pub struct InstructionData {}
