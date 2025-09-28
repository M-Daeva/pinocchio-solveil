use {
    base::{
        traits::DataLen,
        types::{BitField, String4096, Uint32, Uint64},
    },
    bytemuck::{Pod, Zeroable},
    codama::{CodamaInstructions, CodamaType},
    macro_p_serde::p_serde,
    pinocchio::pubkey::Pubkey,
};

#[derive(CodamaType)]
#[p_serde]
pub struct AssetItem {
    pub amount: Uint64,
    pub asset: Pubkey,
}

#[derive(CodamaType)]
#[p_serde]
pub struct Range {
    pub min: Uint32,
    pub max: Uint32,
}

#[derive(CodamaInstructions)]
#[repr(u8)]
pub enum InstructionData {
    #[codama(account(name = "system_program"))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "associated_token_program"))]
    //
    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "bump", writable))]
    #[codama(account(name = "config", writable))]
    #[codama(account(name = "user_counter", writable))]
    #[codama(account(name = "admin_rotation_state", writable))]
    #[codama(account(name = "revenue_mint"))]
    #[codama(account(name = "revenue_app_ata", writable))]
    Init {
        flags: BitField,
        rotation_timeout: Uint32,
        account_registration_fee: AssetItem,
        account_data_size_range: Range,
    },

    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "config", writable))]
    #[codama(account(name = "admin_rotation_state", writable))]
    UpdateConfig {
        flags: BitField,
        admin: Pubkey,
        is_paused: BitField,
        rotation_timeout: Uint32,
        registration_fee_amount: Uint64,
        data_size_range: Range,
    },

    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "config", writable))]
    #[codama(account(name = "admin_rotation_state", writable))]
    ConfirmAdminRotation {},

    #[codama(account(name = "system_program"))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "associated_token_program"))]
    //
    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "recipient", writable))] // handle the option on client
    #[codama(account(name = "bump"))]
    #[codama(account(name = "config"))]
    #[codama(account(name = "revenue_mint"))]
    #[codama(account(name = "revenue_recipient_ata", writable))]
    #[codama(account(name = "revenue_app_ata", writable))]
    WithdrawRevenue { flags: BitField, amount: Uint64 },

    #[codama(account(name = "system_program"))]
    //
    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "bump"))]
    #[codama(account(name = "config"))]
    #[codama(account(name = "user_counter", writable))]
    #[codama(account(name = "user_id", writable))]
    #[codama(account(name = "user_account", writable))]
    #[codama(account(name = "user_rotation_state", writable))]
    CreateAccount { max_data_size: Uint32 },

    #[codama(account(name = "system_program"))]
    //
    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "user_id", writable))]
    #[codama(account(name = "user_account", writable))]
    #[codama(account(name = "user_rotation_state", writable))]
    CloseAccount {},

    #[codama(account(name = "system_program"))]
    //
    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "bump"))]
    #[codama(account(name = "config"))]
    #[codama(account(name = "user_id", writable))]
    #[codama(account(name = "user_account", writable))]
    #[codama(account(name = "user_rotation_state", writable))]
    ReopenAccount { max_data_size: Uint32 },

    #[codama(account(name = "system_program"))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "associated_token_program"))]
    //
    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "bump"))]
    #[codama(account(name = "config"))]
    #[codama(account(name = "user_id", writable))]
    #[codama(account(name = "revenue_mint"))]
    #[codama(account(name = "revenue_sender_ata", writable))]
    #[codama(account(name = "revenue_app_ata", writable))]
    ActivateAccount {},

    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "user_id"))]
    #[codama(account(name = "user_account", writable))]
    WriteData { data: String4096, nonce: Uint64 },

    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "bump"))]
    #[codama(account(name = "config"))]
    #[codama(account(name = "user_id"))]
    #[codama(account(name = "user_rotation_state", writable))]
    RequestAccountRotation { new_owner: Pubkey },

    #[codama(account(name = "system_program"))]
    //
    #[codama(account(name = "sender", signer, writable))]
    #[codama(account(name = "user_id_pre", writable))]
    #[codama(account(name = "user_id", writable))]
    #[codama(account(name = "user_rotation_state", writable))]
    ConfirmAccountRotation {},
}
