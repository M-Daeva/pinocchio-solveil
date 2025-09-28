use {
    crate::helpers::suite::{
        core::{
            extension::{get_data, send_tx_with_ix},
            App, ProgramId,
        },
        types::{
            pin_to_sol_pubkey, sol_to_pin_pubkey, AppToken, AppUser, SolPubkey, TestError,
            TestResult,
        },
    },
    base::{
        traits::InstructionSerialize,
        types::{String4096, Uint32, Uint64},
    },
    litesvm::types::TransactionMetadata,
    registry_cpi::{
        state::{
            Config, RotationState, UserAccount, UserCounter, UserId, ACCOUNT_DATA_SIZE_MAX,
            ACCOUNT_REGISTRATION_FEE_ASSET,
        },
        types::{
            self,
            common::{AssetItem, Range},
        },
    },
};

pub trait RegistryExtension {
    fn registry_try_init(
        &mut self,
        sender: AppUser,
        rotation_timeout: Option<u32>,
        account_registration_fee: Option<AssetItem>,
        account_data_size_range: Option<Range>,
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_update_config(
        &mut self,
        sender: AppUser,
        admin: Option<AppUser>,
        is_paused: Option<bool>,
        rotation_timeout: Option<u32>,
        registration_fee_amount: Option<u64>,
        data_size_range: Option<Range>,
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_confirm_admin_rotation(
        &mut self,
        sender: AppUser,
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_withdraw_revenue(
        &mut self,
        sender: AppUser,
        amount: Option<u64>,
        recipient: Option<AppUser>,
        revenue_asset: Option<AppToken>, // to test guards
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_create_account(
        &mut self,
        sender: AppUser,
        max_data_size: u32,
        expected_user_id: Option<u32>, // to test guards
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_close_account(
        &mut self,
        sender: AppUser,
        user: Option<AppUser>, // to test guards
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_reopen_account(
        &mut self,
        sender: AppUser,
        max_data_size: u32,
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_activate_account(
        &mut self,
        sender: AppUser,
        user: Option<AppUser>,
        revenue_asset: Option<AppToken>, // to test guards
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_write_data(
        &mut self,
        sender: AppUser,
        data: &str,
        nonce: u64,
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_request_account_rotation(
        &mut self,
        sender: AppUser,
        new_owner: AppUser,
    ) -> TestResult<TransactionMetadata>;

    fn registry_try_confirm_account_rotation(
        &mut self,
        sender: AppUser,
        prev_owner: AppUser,
    ) -> TestResult<TransactionMetadata>;

    fn registry_query_config(&self) -> TestResult<Config>;

    fn registry_query_user_counter(&self) -> TestResult<UserCounter>;

    fn registry_query_admin_rotation_state(&self) -> TestResult<RotationState>;

    fn registry_query_user_id(&self, user: AppUser) -> TestResult<UserId>;

    fn registry_query_user_account(&self, user: AppUser) -> TestResult<UserAccount>;

    fn registry_query_user_rotation_state(&self, user: AppUser) -> TestResult<RotationState>;
}

impl RegistryExtension for App {
    fn registry_try_init(
        &mut self,
        sender: AppUser,
        rotation_timeout: Option<u32>,
        account_registration_fee: Option<AssetItem>,
        account_data_size_range: Option<Range>,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            token_program,
            associated_token_program,
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // mint
        let revenue_mint = solana_pubkey::Pubkey::from(
            account_registration_fee
                .as_ref()
                .map(|x| x.asset)
                .unwrap_or(ACCOUNT_REGISTRATION_FEE_ASSET),
        );

        // pda
        let bump = self.pda.registry_bump();
        let config = self.pda.registry_config();
        let user_counter = self.pda.registry_user_counter();
        let admin_rotation_state = self.pda.registry_admin_rotation_state();

        // ata
        let revenue_app_ata = App::get_ata(&config, &revenue_mint);

        let accounts = types::init::TestAccounts {
            system_program,
            token_program,
            associated_token_program,
            sender,
            bump,
            config,
            user_counter,
            admin_rotation_state,
            revenue_mint,
            revenue_app_ata,
        }
        .to_account_metas();

        let mut ix_data = types::init::InstructionData::default();
        ix_data.set_rotation_timeout(rotation_timeout.map(|x| x.into()));
        ix_data.set_account_registration_fee(account_registration_fee);
        ix_data.set_account_data_size_range(account_data_size_range);

        let ix_data = &ix_data.serialize().map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_update_config(
        &mut self,
        sender: AppUser,
        admin: Option<AppUser>,
        is_paused: Option<bool>,
        rotation_timeout: Option<u32>,
        registration_fee_amount: Option<u64>,
        data_size_range: Option<Range>,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // pda
        let config = self.pda.registry_config();
        let admin_rotation_state = self.pda.registry_admin_rotation_state();

        let accounts = types::update_config::TestAccounts {
            sender,
            config,
            admin_rotation_state,
        }
        .to_account_metas();

        let mut ix_data = types::update_config::InstructionData::default();
        ix_data.set_admin(admin.map(|x| sol_to_pin_pubkey(&x.pubkey())));
        ix_data.set_is_paused(is_paused.map(|x| x.into()));
        ix_data.set_rotation_timeout(rotation_timeout.map(|x| x.into()));
        ix_data.set_registration_fee_amount(registration_fee_amount.map(|x| x.into()));
        ix_data.set_data_size_range(data_size_range);

        let ix_data = &ix_data.serialize().map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_confirm_admin_rotation(
        &mut self,
        sender: AppUser,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // pda
        let config = self.pda.registry_config();
        let admin_rotation_state = self.pda.registry_admin_rotation_state();

        // confirm_admin_rotation uses update_config account
        let accounts = types::update_config::TestAccounts {
            sender,
            config,
            admin_rotation_state,
        }
        .to_account_metas();

        let ix_data = &types::confirm_admin_rotation::InstructionData {}
            .serialize()
            .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_withdraw_revenue(
        &mut self,
        sender: AppUser,
        amount: Option<u64>,
        recipient: Option<AppUser>,
        revenue_asset: Option<AppToken>, // to test guards
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            token_program,
            associated_token_program,
            registry: program_id,
            ..
        } = self.program_id;

        let recipient = recipient.unwrap_or(sender).pubkey();

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // mint
        let revenue_mint = match revenue_asset {
            Some(x) => x.pubkey(),
            _ => pin_to_sol_pubkey(&self.registry_query_config()?.registration_fee.asset),
        };

        // pda
        let bump = self.pda.registry_bump();
        let config = self.pda.registry_config();

        // ata
        let revenue_recipient_ata = App::get_ata(&recipient, &revenue_mint);
        let revenue_app_ata = App::get_ata(&config, &revenue_mint);

        let accounts = types::withdraw_revenue::TestAccounts {
            system_program,
            token_program,
            associated_token_program,
            sender,
            recipient,
            bump,
            config,
            revenue_mint,
            revenue_recipient_ata,
            revenue_app_ata,
        }
        .to_account_metas();

        let mut ix_data = types::withdraw_revenue::InstructionData::default();
        if let Some(x) = amount {
            ix_data.flags.set_bit(true);
            ix_data.amount.set(x);
        }

        let ix_data = &ix_data.serialize().map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_create_account(
        &mut self,
        sender: AppUser,
        max_data_size: u32,
        expected_user_id: Option<u32>, // to test guards
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // pda
        let bump = self.pda.registry_bump();
        let config = self.pda.registry_config();
        let user_counter = self.pda.registry_user_counter();

        let user_id = self.pda.registry_user_id(sender);
        let expected_user_id =
            expected_user_id.unwrap_or(self.registry_query_user_counter()?.last_user_id.get() + 1);
        let user_account = self.pda.registry_user_account(expected_user_id);
        let user_rotation_state = self.pda.registry_user_rotation_state(expected_user_id);

        let accounts = types::create_account::TestAccounts {
            system_program,
            sender,
            bump,
            config,
            user_counter,
            user_id,
            user_account,
            user_rotation_state,
        }
        .to_account_metas();

        let ix_data = &types::create_account::InstructionData {
            max_data_size: Uint32::from(max_data_size),
        }
        .serialize()
        .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_close_account(
        &mut self,
        sender: AppUser,
        user: Option<AppUser>, // to test guards
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            registry: program_id,
            ..
        } = self.program_id;

        let user = user.unwrap_or(sender);

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // pda
        let user_id = self.pda.registry_user_id(user.pubkey());
        let id = self.registry_query_user_id(user)?.id;
        let user_account = self.pda.registry_user_account(id.get());
        let user_rotation_state = self.pda.registry_user_rotation_state(id.get());

        let accounts = types::close_account::TestAccounts {
            system_program,
            sender,
            user_id,
            user_account,
            user_rotation_state,
        }
        .to_account_metas();

        let ix_data = &types::close_account::InstructionData {}
            .serialize()
            .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_reopen_account(
        &mut self,
        sender: AppUser,
        max_data_size: u32,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let payer = sender.pubkey();

        // pda
        let bump = self.pda.registry_bump();
        let config = self.pda.registry_config();

        let user_id = self.pda.registry_user_id(payer);
        let id = self.registry_query_user_id(sender)?.id;
        let user_account = self.pda.registry_user_account(id.get());
        let user_rotation_state = self.pda.registry_user_rotation_state(id.get());

        let accounts = types::reopen_account::TestAccounts {
            system_program,
            sender: payer,
            bump,
            config,
            user_id,
            user_account,
            user_rotation_state,
        }
        .to_account_metas();

        let ix_data = &types::reopen_account::InstructionData {
            max_data_size: Uint32::from(max_data_size),
        }
        .serialize()
        .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_activate_account(
        &mut self,
        sender: AppUser,
        user: Option<AppUser>,
        revenue_asset: Option<AppToken>, // to test guards
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            token_program,
            associated_token_program,
            registry: program_id,
            ..
        } = self.program_id;

        let user = user.unwrap_or(sender).pubkey();

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // mint
        let revenue_mint = match revenue_asset {
            Some(x) => x.pubkey(),
            _ => pin_to_sol_pubkey(&self.registry_query_config()?.registration_fee.asset),
        };

        // pda
        let bump = self.pda.registry_bump();
        let config = self.pda.registry_config();
        let user_id = self.pda.registry_user_id(user);

        // ata
        let revenue_sender_ata = App::get_ata(&sender, &revenue_mint);
        let revenue_app_ata = App::get_ata(&config, &revenue_mint);

        let accounts = types::activate_account::TestAccounts {
            system_program,
            token_program,
            associated_token_program,
            sender,
            bump,
            config,
            user_id,
            revenue_mint,
            revenue_sender_ata,
            revenue_app_ata,
        }
        .to_account_metas();

        let ix_data = &types::activate_account::InstructionData {}
            .serialize()
            .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_write_data(
        &mut self,
        sender: AppUser,
        data: &str,
        nonce: u64,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let payer = sender.pubkey();

        // pda
        let user_id = self.pda.registry_user_id(payer);
        let id = self.registry_query_user_id(sender)?.id;
        let user_account = self.pda.registry_user_account(id.get());

        let accounts = types::write_data::TestAccounts {
            sender: payer,
            user_id,
            user_account,
        }
        .to_account_metas();

        let ix_data = &types::write_data::InstructionData {
            data: String4096::from(data),
            nonce: Uint64::from(nonce),
        }
        .serialize()
        .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_request_account_rotation(
        &mut self,
        sender: AppUser,
        new_owner: AppUser,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let payer = sender.pubkey();

        // pda
        let bump = self.pda.registry_bump();
        let config = self.pda.registry_config();

        let user_id = self.pda.registry_user_id(payer);
        let id = self.registry_query_user_id(sender)?.id;
        let user_rotation_state = self.pda.registry_user_rotation_state(id.get());

        let accounts = types::request_account_rotation::TestAccounts {
            sender: payer,
            bump,
            config,
            user_id,
            user_rotation_state,
        }
        .to_account_metas();

        let ix_data = &types::request_account_rotation::InstructionData {
            new_owner: sol_to_pin_pubkey(&new_owner.pubkey()),
        }
        .serialize()
        .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_try_confirm_account_rotation(
        &mut self,
        sender: AppUser,
        prev_owner: AppUser,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            registry: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let payer = sender.pubkey();

        // pda
        let user_id_pre = self.pda.registry_user_id(prev_owner.pubkey());
        let user_id = self.pda.registry_user_id(payer);
        let user_id_value_pre = self.registry_query_user_id(prev_owner)?.id;
        let user_rotation_state = self
            .pda
            .registry_user_rotation_state(user_id_value_pre.get());

        let accounts = types::confirm_account_rotation::TestAccounts {
            system_program,
            sender: payer,
            user_id_pre,
            user_id,
            user_rotation_state,
        }
        .to_account_metas();

        let ix_data = &types::confirm_account_rotation::InstructionData {}
            .serialize()
            .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(self, &program_id, &accounts, &ix_data, signers, &[])
    }

    fn registry_query_config(&self) -> TestResult<Config> {
        get_data(&self.litesvm, &self.pda.registry_config())
    }

    fn registry_query_user_counter(&self) -> TestResult<UserCounter> {
        get_data(&self.litesvm, &self.pda.registry_user_counter())
    }

    fn registry_query_admin_rotation_state(&self) -> TestResult<RotationState> {
        get_data(&self.litesvm, &self.pda.registry_admin_rotation_state())
    }

    fn registry_query_user_id(&self, user: AppUser) -> TestResult<UserId> {
        get_data(&self.litesvm, &self.pda.registry_user_id(user.pubkey()))
    }

    fn registry_query_user_account(&self, user: AppUser) -> TestResult<UserAccount> {
        let user_id = self.registry_query_user_id(user)?;
        get_data(
            &self.litesvm,
            &self.pda.registry_user_account(user_id.id.get()),
        )
    }

    fn registry_query_user_rotation_state(&self, user: AppUser) -> TestResult<RotationState> {
        let user_id = self.registry_query_user_id(user)?;
        get_data(
            &self.litesvm,
            &self.pda.registry_user_rotation_state(user_id.id.get()),
        )
    }
}

pub fn get_data_buffer(data: &str) -> [u8; ACCOUNT_DATA_SIZE_MAX as usize] {
    let mut buffer = [0u8; ACCOUNT_DATA_SIZE_MAX as usize];
    let bytes = data.as_bytes();
    buffer[..bytes.len()].copy_from_slice(bytes);
    buffer
}
