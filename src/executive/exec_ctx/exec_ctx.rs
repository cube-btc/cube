use crate::constructive::bitcoiny::batch_template::batch_template::BatchTemplate;
use crate::constructive::entry::entry::entry::Entry;
use crate::executive::exec_ctx::errors::batch_execution_error::BatchExecutionError;
use crate::executive::exec_ctx::errors::into_batch_template_error::IntoBatchTemplateError;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::transmutative::codec::bitvec_ext::BitVecExt;
use crate::{
    constructive::entry::entry_kinds::liftup::liftup::Liftup,
    executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError,
    executive::exec_ctx::errors::apply_changes_error::ApplyChangesError,
};
use bit_vec::BitVec;
use bitcoin::{OutPoint, TxOut};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::constructive::core_types::valtypes::val::long_val::long_val::LongVal;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

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
    pub async fn pre_execution(&mut self) {
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
    pub async fn rollback_last(&mut self) {
        // 1 Rollback last coin manager.
        self.coin_manager.lock().await.rollback_last();

        // 2 Rollback last graveyard.
        self.graveyard.lock().await.rollback_last();

        // 3 Rollback last registery.
        self.registery.lock().await.rollback_last();

        // 4 Rollback last flame manager.
        self.flame_manager.lock().await.rollback_last();
    }

    /// Flushes all the changes in the `ExecCtx`.
    pub async fn flush(&mut self) {
        // 1 Rollback all coin manager.
        self.coin_manager.lock().await.flush_delta();

        // 2 Rollback all graveyard.
        self.graveyard.lock().await.flush_deltas();

        // 3 Rollback all registery.
        self.registery.lock().await.flush_delta();

        // 4 Rollback all flame manager.
        self.flame_manager.lock().await.flush_delta();

        // 5 Reset the added tx inputs.
        self.added_tx_inputs.clear();

        // 6 Reset the added tx outputs.
        self.added_tx_outputs.clear();

        // 7 Reset the executed entries.
        self.executed_entries.clear();
    }

    /// Applies the changes to the `ExecCtx` collectively for all Entries in the container.
    async fn _apply_changes(
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
        self.flush().await;

        // 6 Return Ok.
        Ok(())
    }

    /// Converts the `ExecCtx` into a `BatchTemplate`.
    ///
    /// Used by Engine to convert the `ExecCtx` in its `SessionPool` into a `BatchTemplate`.
    pub async fn into_batch_template(
        &mut self,
        batch_height: u64,
        batch_timestamp: u64,
        payload_version: u32,
    ) -> Result<BatchTemplate, IntoBatchTemplateError> {
        // 1 Initialize the bit vector for the payload.
        let mut payload_bits: BitVec = BitVec::new();

        // 2 Encode the payload version.
        {
            // 2.1 Encode the payload version.
            let payload_version_bits = ShortVal::new(payload_version).encode_ape();

            // 2.2 Extend the payload bits with the payload version bits.
            payload_bits.extend(payload_version_bits);
        }

        // 3 Encode the batch timestamp.
        {
            // 3.1 Encode the batch timestamp.
            let batch_timestamp_bits = LongVal::new(batch_timestamp).encode_ape();

            // 3.2 Extend the payload bits with the batch timestamp bits.
            payload_bits.extend(batch_timestamp_bits);
        }

        // 4 Iterator over the executed entries to extend the payload bits.
        for entry in &self.executed_entries {
            // 4.1 Encode the entry as APE bits.
            let entry_ape_bits = entry
                .encode_ape(batch_height, &self.registery, true, true)
                .await
                .map_err(IntoBatchTemplateError::EntryAPEEncodeError)?;

            // 4.2 Extend the payload bits with the entry APE bits.
            payload_bits.extend(entry_ape_bits);
        }

        // 5 Convert the payload bits to payload bytes.
        let payload_bytes: Bytes = payload_bits.to_payload_bytes();

        // 6 Collect the Bitcoin transaction inputs.
        let bitcoin_tx_inputs: Vec<OutPoint> = self
            .added_tx_inputs
            .iter()
            .map(|(outpoint, _)| outpoint.clone())
            .collect();

        // 7 Get the Bitcoin transaction outputs.
        let bitcoin_tx_outputs: Vec<TxOut> = self.added_tx_outputs.clone();

        // 8 Construct the batch template.
        let batch_template =
            BatchTemplate::new(bitcoin_tx_inputs, bitcoin_tx_outputs, payload_bytes);

        // 9 Return the batch template.
        Ok(batch_template)
    }

    /// Executes a batch of Entries from a `BatchTemplate`.
    ///
    /// Used by Nodes to execute a batch of Entries from a `BatchTemplate`.
    pub async fn execute_batch(
        &mut self,
        batch_height: u64,
        batch_template: BatchTemplate,
    ) -> Result<(), BatchExecutionError> {
        // 1 Get the batch payload as bits.
        let batch_payload = batch_template.payload_bits().ok_or(
            BatchExecutionError::BatchTemplatePayloadBitsConversionError(
                batch_template.payload_bytes.clone(),
            ),
        )?;

        // 2 Turn the APE payload into an iterator.
        let mut ape_bitstream = batch_payload.iter();

        // 3 Turn the Bitcoin transaction inputs into an iterator.
        let mut tx_inputs_iter = batch_template.bitcoin_tx_inputs.into_iter();

        // 4 Turn the Bitcoin transaction outputs into an iterator.
        let mut _tx_outputs_iter = batch_template.bitcoin_tx_outputs.iter();

        // 5 Return params from the params manager: TODO
        let decode_account_rank_as_longval = true;
        let decode_contract_rank_as_longval = true;
        let base_ops_price = 100;

        // 6 Decode payload version as as shortcal and session timestamp as a longval.
        let _payload_version: u32 = ShortVal::decode_ape(&mut ape_bitstream)
            .map_err(BatchExecutionError::DecodePayloadVersionError)?
            .value();

        // 7 Decode batch timestamp as a longval used as the execution timestamp for all entries.
        let batch_timestamp: u64 = LongVal::decode_ape(&mut ape_bitstream)
            .map_err(BatchExecutionError::DecodeBatchTimestampError)?
            .value();

        // 8 Decode entries from the patload one by one and execute them.
        loop {
            // 8.1 Break out of the loop if the APE bitstream is empty.
            if ape_bitstream.next().is_none() {
                break;
            }

            // 8.2 Decode Entry from the APE bitstream.
            let entry = Entry::decode_ape(
                self.engine_key,
                batch_height,
                &mut ape_bitstream,
                &mut tx_inputs_iter,
                decode_account_rank_as_longval,
                decode_contract_rank_as_longval,
                base_ops_price,
                &self.utxo_set,
                &self.registery,
            )
            .await
            .map_err(BatchExecutionError::DecodeEntryError)?;

            // 8.3 Execute the decoded `Entry`.
            match entry {
                // 8.3.a The `Entry` is a `Liftup`.
                Entry::Liftup(liftup) => {
                    // 8.3.a.1 Execute the `Liftup` `Entry`.
                    if let Err(error) = self.execute_liftup(liftup, batch_timestamp).await {
                        return Err(BatchExecutionError::LiftupExecutionError(error));
                    }
                }
                _ => panic!("Not implemented yet."),
            }
        }

        // 9 Return Ok.
        Ok(())
    }

    /// Executes a `Liftup` Entry.
    pub async fn execute_liftup(
        &mut self,
        liftup: Liftup,
        execution_timestamp: u64,
    ) -> Result<Entry, LiftupExecutionError> {
        // 1 Execute the liftup.
        match self
            .execute_liftup_internal(&liftup, execution_timestamp)
            .await
        {
            // 1.a Success.
            Ok(_) => {
                // 1.a.1 Add Lifts inside the Liftup to the added tx inputs.
                self.added_tx_inputs.extend(
                    liftup
                        .lift_prevtxos
                        .iter()
                        .map(|lift| (lift.outpoint(), lift.txout().clone())),
                );

                // 1.a.2 Construct the Liftup entry.
                let liftup_entry = Entry::new_liftup(liftup);

                // 1.a.3 Add the liftup entry to the executed entries.
                self.executed_entries.push(liftup_entry.clone());

                // 1.a.4 Return Ok.
                Ok(liftup_entry)
            }
            // 1.b Error.
            Err(error) => Err(error),
        }
    }
}
