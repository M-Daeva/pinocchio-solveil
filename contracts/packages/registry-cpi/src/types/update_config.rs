use {
    crate::{state::Discriminator, types::common::Range},
    base::{
        traits::DataLen,
        types::{BitField, Result, Uint32, Uint64},
    },
    bytemuck::{Pod, Zeroable},
    codama::CodamaInstruction,
    macro_optional_flag::OptionFlag,
    macro_p_serde::p_serde,
    macro_test_ser::test_ser,
    macro_try_from::AccountTryFrom,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
    r#macro_account::AccountMetas,
};

// TODO: fix warning: unused imports: `bool_as_bytes`, `pubkey_as_bytes`, `state::discriminator as DISCRIMINATOR`, and `u64_as_bytes`
// related to macro usage

#[derive(AccountTryFrom, AccountMetas)]
pub struct Accounts<'a> {
    #[account(signer, writable)]
    pub sender: &'a AccountInfo,

    #[account(writable)]
    pub config: &'a AccountInfo,

    #[account(writable)]
    pub admin_rotation_state: &'a AccountInfo,
}

#[derive(CodamaInstruction, OptionFlag)]
// #[codama(name = "UpdateConfig")]
#[test_ser(Discriminator::UpdateConfig)]
#[p_serde]
pub struct InstructionData {
    #[optional(
        _is_paused,
        admin,
        rotation_timeout,
        registration_fee_amount,
        data_size_range
    )]
    pub flags: BitField,
    pub admin: Pubkey,
    pub rotation_timeout: Uint32,
    pub registration_fee_amount: Uint64,
    pub data_size_range: Range,
}

// impl InstructionData {
//     const _IS_PAUSED: u8 = 0; // 1 bit for optionality
//     const IS_PAUSED: u8 = 1; // 1 bit for value
//     const ADMIN: u8 = 2;

//     #[inline]
//     fn get_is_paused_flag(&self) -> bool {
//         self.flags.get_flag(Self::_IS_PAUSED)
//     }

//     #[inline]
//     fn set_is_paused_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::_IS_PAUSED, x);
//     }

//     #[inline]
//     pub fn get_is_paused(&self) -> Option<bool> {
//         if self.get_is_paused_flag() {
//             Some(self.flags.get_flag(Self::IS_PAUSED))
//         } else {
//             None
//         }
//     }

//     #[inline]
//     pub fn set_is_paused(&mut self, x: Option<bool>) {
//         match x {
//             Some(x) => {
//                 self.set_is_paused_flag(true);
//                 self.flags.set_flag(Self::IS_PAUSED, x);
//             }
//             None => {
//                 self.set_is_paused_flag(false);
//             }
//         }
//     }

//     #[inline]
//     fn get_admin_flag(&self) -> bool {
//         self.flags.get_flag(Self::ADMIN)
//     }

//     #[inline]
//     fn set_admin_flag(&mut self, x: bool) {
//         self.flags.set_flag(Self::ADMIN, x);
//     }

//     #[inline]
//     pub fn get_admin(&self) -> Option<Pubkey> {
//         if self.get_admin_flag() {
//             Some(self.admin)
//         } else {
//             None
//         }
//     }

//     #[inline]
//     pub fn set_admin(&mut self, x: Option<Pubkey>) {
//         match x {
//             Some(x) => {
//                 self.set_admin_flag(true);
//                 self.admin = x;
//             }
//             None => {
//                 self.set_admin_flag(false);
//             }
//         }
//     }
// }
