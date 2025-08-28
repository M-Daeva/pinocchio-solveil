use {
    crate::helpers::suite::{
        core::{
            extension::{get_data, send_tx_with_ix},
            App, ProgramId,
        },
        types::{AppUser, SolPubkey, TestError, TestResult},
    },
    base::types::InstructionSerialize,
    litesvm::types::TransactionMetadata,
    registry_cpi::{
        state::{Config, ACCOUNT_REGISTRATION_FEE_ASSET},
        types::{
            self,
            common::{AssetItem, Range},
        },
    },
};

pub trait CounterExtension {
    fn registry_try_init(
        &mut self,
        sender: AppUser,
        rotation_timeout: Option<u32>,
        account_registration_fee: Option<AssetItem>,
        account_data_size_range: Option<Range>,
    ) -> TestResult<TransactionMetadata>;

    fn registry_query_config(&self) -> TestResult<Config>;
}

impl CounterExtension for App {
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

        let instruction_data = &types::init::InstructionData {
            rotation_timeout,
            account_registration_fee,
            account_data_size_range,
        }
        .serialize()
        .map_err(TestError::from_raw_error)?;

        send_tx_with_ix(
            self,
            &program_id,
            &accounts,
            &instruction_data,
            signers,
            &[],
        )
    }

    fn registry_query_config(&self) -> TestResult<Config> {
        get_data(&self.litesvm, &self.pda.registry_config(), 0)
    }
}
