use {
    crate::state::Discriminator,
    base::types::Result,
    bytemuck::{Pod, Zeroable},
    macro_test_ser::test_serialize,
    macro_zc_serde::p_serde,
};

pub use crate::types::update_config::Accounts;

#[test_serialize(Discriminator::ConfirmAdminRotation)]
#[p_serde]
pub struct InstructionData {}
