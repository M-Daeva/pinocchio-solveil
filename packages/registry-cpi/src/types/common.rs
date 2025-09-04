use {
    base::{
        traits::DataLen,
        types::{Uint32, Uint64},
    },
    bytemuck::{Pod, Zeroable},
    macro_p_serde::p_serde,
    pinocchio::pubkey::Pubkey,
};

#[p_serde]
pub struct AssetItem {
    pub amount: Uint64,
    pub asset: Pubkey,
}

#[p_serde]
pub struct Range {
    pub min: Uint32,
    pub max: Uint32,
}
