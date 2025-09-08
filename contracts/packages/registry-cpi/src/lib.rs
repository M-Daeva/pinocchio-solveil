use pinocchio_pubkey::declare_id;

pub mod error;
pub mod state;

pub mod types {
    pub mod activate_account;
    pub mod close_account;
    pub mod common;
    pub mod confirm_account_rotation;
    pub mod confirm_admin_rotation;
    pub mod create_account;
    pub mod init;
    pub mod reopen_account;
    pub mod request_account_rotation;
    pub mod update_config;
    pub mod withdraw_revenue;
    pub mod write_data;
}

declare_id!("89KoDhPxWcegVeGrr8sAg3sn7H7EaH6edtDg9qx8Jh19");
