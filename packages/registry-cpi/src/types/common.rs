use {
    bytemuck::{Pod, Zeroable},
    pinocchio::pubkey::Pubkey,
};

#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct AssetItem {
    pub amount: [u8; 8],
    pub asset: Pubkey,
}

impl AssetItem {
    #[inline]
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline]
    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }
}

#[derive(Default, Debug, PartialEq, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Range {
    min: [u8; 4],
    max: [u8; 4],
}

impl Range {
    #[inline]
    pub fn min(&self) -> u32 {
        u32::from_le_bytes(self.min)
    }

    #[inline]
    pub fn set_min(&mut self, min: u32) {
        self.min = min.to_le_bytes();
    }

    #[inline]
    pub fn max(&self) -> u32 {
        u32::from_le_bytes(self.max)
    }

    #[inline]
    pub fn set_max(&mut self, max: u32) {
        self.max = max.to_le_bytes();
    }
}
