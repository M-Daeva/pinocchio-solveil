use pinocchio_pubkey::declare_id;

pub mod error;
pub mod state;

pub mod types {
    pub mod activate_account;
    pub mod common;
    pub mod confirm_admin_rotation;
    pub mod create_account;
    pub mod init;
    pub mod update_config;
    pub mod withdraw_revenue;
}

declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");
