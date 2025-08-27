use {
    crate::helpers::suite::{
        core::{
            extension::{get_data, send_tx_with_ix},
            App, ProgramId,
        },
        types::{AppUser, TestError, TestResult},
    },
    base::types::InstructionSerialize,
    counter_cpi::{state::Counter, types},
    litesvm::types::TransactionMetadata,
};

pub trait CounterExtension {
    fn counter_try_init(
        &mut self,
        sender: AppUser,
        value: Option<u8>,
    ) -> TestResult<TransactionMetadata>;

    fn counter_try_set(&mut self, sender: AppUser, value: u8) -> TestResult<TransactionMetadata>;

    fn counter_query_counter(&self) -> TestResult<Counter>;
}

impl CounterExtension for App {
    fn counter_try_init(
        &mut self,
        sender: AppUser,
        value: Option<u8>,
    ) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            system_program,
            counter: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // pda
        let counter_account = self.pda.counter_counter();

        let accounts = types::init::TestAccounts {
            sender,
            counter_account,
            system_program,
        }
        .to_account_metas();

        let instruction_data = &types::init::InstructionData { value }
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

    fn counter_try_set(&mut self, sender: AppUser, value: u8) -> TestResult<TransactionMetadata> {
        // programs
        let ProgramId {
            counter: program_id,
            ..
        } = self.program_id;

        // signers
        let signers = &[sender.keypair()];
        let sender = sender.pubkey();

        // pda
        let counter_account = self.pda.counter_counter();

        let accounts = types::set::TestAccounts {
            sender,
            counter_account,
        }
        .to_account_metas();

        let instruction_data = &types::set::InstructionData { value }
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

    fn counter_query_counter(&self) -> TestResult<Counter> {
        get_data(&self.litesvm, &self.pda.counter_counter())
    }
}
