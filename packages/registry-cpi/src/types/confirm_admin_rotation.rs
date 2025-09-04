use {
    crate::state::Discriminator,
    base::types::Result,
    bytemuck::{Pod, Zeroable},
    macro_p_serde::p_serde,
    macro_test_ser::test_serialize,
};

pub use crate::types::update_config::Accounts;

#[test_serialize(Discriminator::ConfirmAdminRotation)]
#[p_serde]
pub struct InstructionData {}
