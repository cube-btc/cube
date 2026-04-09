use crate::constructive::entry::entry::Entry;
use crate::executive::exec_ctx::errors::batch_execution_error::BatchExecutionError;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::{
    constructive::entry::entry_types::liftup::liftup::Liftup,
    executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError,
    executive::exec_ctx::errors::apply_changes_error::ApplyChangesError,
};
use bit_vec::BitVec;
use bitcoin::{OutPoint, TxOut};
use std::sync::Arc;
use tokio::sync::Mutex;

/// `ExecCtx` contains a set of executed entries.
pub struct ExecCtx {
    // The key of the Engine.
    pub engine_key: [u8; 32],

    // The local utxo set database of the Engine.
    pub utxo_set: UTXO_SET,

    // The local registery database of the Engine.
    pub registery: REGISTERY,

    // The local graveyard database of the Engine.
    pub graveyard: GRAVEYARD,

    // The local coin manager database of the Engine.
    pub coin_manager: COIN_MANAGER,

    // The local flame manager database of the Engine.
    pub flame_manager: FLAME_MANAGER,

    // The entries that have been executed in the pool.
    pub executed_entries: Vec<Entry>,

    // The Bitcoin transaction inputs that have been added as a result of executing `Liftup` entries.
    pub added_tx_inputs: Vec<(OutPoint, TxOut)>,

    // The Bitcoin transaction outputs that have been added as a result of executing `Swapout` entries.
    pub added_tx_outputs: Vec<TxOut>,
}

/// Guarded `ExecCtx`.
#[allow(non_camel_case_types)]
pub type EXEC_CTX = Arc<Mutex<ExecCtx>>;

impl ExecCtx {
    /// Constructs the `ExecCtx`.
    pub fn construct(
        engine_key: [u8; 32],
        utxo_set: UTXO_SET,
        registery: REGISTERY,
        graveyard: GRAVEYARD,
        coin_manager: COIN_MANAGER,
        flame_manager: FLAME_MANAGER,
    ) -> EXEC_CTX {
        // 1 Initialize the `ExecCtx`.
        let exec_ctx = ExecCtx {
            engine_key,
            utxo_set,
            registery,
            graveyard,
            coin_manager,
            flame_manager,
            executed_entries: Vec::new(),
            added_tx_inputs: Vec::new(),
            added_tx_outputs: Vec::new(),
        };

        // 2 Return the guarded `ExecCtx`.
        Arc::new(Mutex::new(exec_ctx))
    }

    /// Prepares the `ExecCtx` prior to each execution.
    async fn pre_execution(&mut self) {
        // 1 Pre-execution coin manager.
        self.coin_manager.lock().await.pre_execution();

        // 2 Pre-execution graveyard.
        self.graveyard.lock().await.pre_execution();

        // 3 Pre-execution registery.
        self.registery.lock().await.pre_execution();

        // 4 Pre-execution flame manager.
        self.flame_manager.lock().await.pre_execution();
    }

    /// Rolls back the last execution of the `ExecCtx` due to a failed individual Entry execution.
    async fn rollback_last(&mut self) {
        // 1 Rollback last coin manager.
        self.coin_manager.lock().await.rollback_last();

        // 2 Rollback last graveyard.
        self.graveyard.lock().await.rollback_last();

        // 3 Rollback last registery.
        self.registery.lock().await.rollback_last();

        // 4 Rollback last flame manager.
        self.flame_manager.lock().await.rollback_last();
    }

    /// Rolls back all the changes in the `ExecCtx`.
    async fn rollback_all(&mut self) {
        // 1 Rollback all coin manager.
        self.coin_manager.lock().await.flush_delta();

        // 2 Rollback all graveyard.
        self.graveyard.lock().await.flush_deltas();

        // 3 Rollback all registery.
        self.registery.lock().await.flush_delta();

        // 4 Rollback all flame manager.
        self.flame_manager.lock().await.flush_delta();
    }

    /// Applies the changes to the `ExecCtx` collectively for all Entries in the container.
    async fn apply_changes(
        &mut self,
        new_projector_height: u64,
        projector_expiry_height: u64,
    ) -> Result<(), ApplyChangesError> {
        // 1 Apply changes to the coin manager.
        if let Err(error) = self.coin_manager.lock().await.apply_changes() {
            return Err(ApplyChangesError::CoinManagerApplyChangesError(error));
        }

        // 2 Apply changes to the graveyard.
        if let Err(error) = self.graveyard.lock().await.apply_changes() {
            return Err(ApplyChangesError::GraveyardApplyChangesError(error));
        }

        // 3 Apply changes to the registery.
        if let Err(error) = self.registery.lock().await.apply_changes() {
            return Err(ApplyChangesError::RegisteryApplyChangesError(error));
        }

        // 4 Apply changes to the flame manager.
        if let Err(error) = self
            .flame_manager
            .lock()
            .await
            .apply_changes(
                &self.coin_manager,
                &self.registery,
                new_projector_height,
                projector_expiry_height,
            )
            .await
        {
            return Err(ApplyChangesError::FlameManagerApplyChangesError(error));
        }

        // 5 Flush the container changes.
        self.rollback_all().await;

        // 6 Return Ok.
        Ok(())
    }

    /// Executes a batch of Entries.
    pub async fn execute_batch(
        &mut self,
        batch_payload: BitVec,
        batch_tx_inputs: Vec<OutPoint>,
        batch_tx_outputs: Vec<TxOut>,
        session_timestamp: u64,
    ) -> Result<(), BatchExecutionError> {
        // 1 Turn the APE payload into an iterator.
        let mut ape_bitstream = batch_payload.iter();

        // 2 Turn the tx inputs into an iterator.
        let mut tx_inputs_iter = batch_tx_inputs.into_iter();

        // 3 Turn the tx outputs into an iterator.
        let mut _tx_outputs_iter = batch_tx_outputs.iter();

        // TODO: These will come from the params manager.
        let decode_account_rank_as_longval = true;
        let decode_contract_rank_as_longval = true;
        let base_ops_price = 100;

        // 4 Decode entries from the patload one by one and execute them.
        loop {
            // 4.1 Check if the APE bitstream is empty.
            if ape_bitstream.next().is_none() {
                break;
            }

            // 4.2 Decode Entry from the APE bitstream.
            let entry = Entry::decode_ape(
                &mut ape_bitstream,
                &mut tx_inputs_iter,
                self.engine_key,
                decode_account_rank_as_longval,
                decode_contract_rank_as_longval,
                base_ops_price,
                &self.utxo_set,
                &self.registery,
            )
            .await
            .map_err(BatchExecutionError::DecodeEntryError)?;

            // 4.3 Match on the Entry type.
            match entry {
                // 4.3.a The Entry is a `Liftup`.
                Entry::Liftup(liftup) => {
                    // 4.3.a.1 Execute the `Liftup` Entry.
                    self.execute_liftup(liftup, session_timestamp)
                        .await
                        .map_err(BatchExecutionError::LiftupExecutionError)?;
                }
                _ => panic!("Not implemented yet."),
            }
        }

        // 5 Return Ok.
        Ok(())
    }

    /// Executes a `Liftup` entry in the pool.
    pub async fn execute_liftup(
        &mut self,
        liftup: Liftup,
        session_timestamp: u64,
    ) -> Result<(), LiftupExecutionError> {
        // 1 Pre-execution.
        self.pre_execution().await;

        // 2 Execute the liftup.
        match self
            .execute_liftup_internal(&liftup, session_timestamp)
            .await
        {
            // 2.a Success.
            Ok(_) => {
                // 2.a.1 Add Lifts inside the Liftup to the added tx inputs.
                self.added_tx_inputs.extend(
                    liftup
                        .lift_prevtxos
                        .iter()
                        .map(|lift| (lift.outpoint(), lift.txout().clone())),
                );

                // 2.a.2 Construct the Liftup entry.
                let liftup_entry = Entry::new_liftup(liftup);

                // 2.a.3 Add the liftup entry to the executed entries.
                self.executed_entries.push(liftup_entry);

                // 2.a.4 Return Ok.
                Ok(())
            }
            // 2.b Error.
            Err(error) => {
                // 2.b.1 Rollback last.
                self.rollback_last().await;

                // 2.b.2 Return the error.
                Err(error)
            }
        }
    }
}
