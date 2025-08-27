use pinocchio::pubkey::Pubkey;

#[derive(Debug, PartialEq)]
pub struct AssetItem {
    pub amount: u64,
    pub asset: Pubkey,
}

#[derive(Debug, PartialEq)]
pub struct Range {
    pub min: u32,
    pub max: u32,
}
