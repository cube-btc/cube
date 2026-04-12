use crate::constructive::bitcoiny::batch_template::batch_template::BatchTemplate;
use crate::constructive::core_types::valtypes::val::long_val::long_val::LongVal;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;
use crate::constructive::entry::entry::entry::Entry;
use crate::executive::exec_ctx::errors::batch_execution_error::BatchExecutionError;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::transmutative::bls::verify::bls_verify_aggregate;
use crate::{
    constructive::entry::entry_kinds::liftup::liftup::Liftup,
    executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError,
    executive::exec_ctx::errors::apply_changes_error::ApplyChangesError,
};
use bit_vec::BitVec;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A type alias for the batch height.
type BatchHeight = u64;

/// A type alias for the payload version.
type PayloadVersion = u32;

/// A type alias for the batch timestamp.
type BatchTimestamp = u64;

/// A type alias for the aggregate BLS signature.
type AggregateBLSSignature = [u8; 96];

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
    }

    /// Applies the changes to the `ExecCtx` collectively for all Entries in the container.
    pub async fn apply_changes(
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

    /// Executes a batch of Entries from a `BatchTemplate`.
    ///
    /// Used by Nodes to execute a batch of Entries from a `BatchTemplate`.
    pub async fn execute_batch(
        &mut self,
        batch_height: u64,
        batch_template: BatchTemplate,
    ) -> Result<
        (
            BatchHeight,
            PayloadVersion,
            BatchTimestamp,
            AggregateBLSSignature,
            Vec<Entry>,
        ),
        BatchExecutionError,
    > {
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
        let payload_version: u32 = ShortVal::decode_ape(&mut ape_bitstream)
            .map_err(BatchExecutionError::DecodePayloadVersionError)?
            .value();

        println!("BE Payload version: {}", payload_version);

        // 7 Decode batch timestamp as a longval used as the execution timestamp for all entries.
        let batch_timestamp: u64 = LongVal::decode_ape(&mut ape_bitstream)
            .map_err(BatchExecutionError::DecodeBatchTimestampError)?
            .value();

        println!("BE Batch timestamp: {}", batch_timestamp);

        // 8 Decode aggregate BLS signature as a byte array.
        let aggregate_bls_signature: [u8; 96] = {
            // 8.1 Collect 768 bits from the APE bitstream.
            let aggregate_bls_signature_bits: BitVec = ape_bitstream.by_ref().take(768).collect();
            if aggregate_bls_signature_bits.len() != 768 {
                return Err(BatchExecutionError::DecodeAggregateBLSSignatureError);
            }

            // 8.2 Convert the aggregate BLS signature bits to a byte array.
            let bytes = aggregate_bls_signature_bits.to_bytes();
            bytes
                .try_into()
                .map_err(|_| BatchExecutionError::DecodeAggregateBLSSignatureError)?
        };

        println!(
            "BE Aggregate BLS signature: {}",
            hex::encode(aggregate_bls_signature)
        );

        // 9 Initialize the executed entries list to collect the executed entries.
        let mut executed_entries: Vec<Entry> = Vec::new();

        // 10 Initialize the executed entry sighashes list.
        let mut executed_entry_sighashes: Vec<[u8; 32]> = Vec::new();

        // 11 Initialize the executed entry account BLS keys list.
        let mut executed_entry_account_bls_keys: Vec<[u8; 48]> = Vec::new();

        // 12 Decode entries from the patload one by one and execute them.
        loop {
            // 12.2 Decode Entry from the APE bitstream.
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

            // 12.3 Execute the decoded `Entry`.
            match entry {
                // 12.3.a The `Entry` is a `Liftup`.
                Entry::Liftup(liftup) => {
                    // 12.3.a.1 Execute the `Liftup` `Entry`.
                    match self.execute_liftup(&liftup, batch_timestamp).await {
                        // 12.3.a.1.a Success.
                        Ok(entry) => {
                            // 12.3.a.1.a.1 Add the liftup entry to the executed entries.
                            executed_entries.push(entry);

                            // 12.3.a.1.a.2 Add the sighash of the `Liftup`.
                            {
                                let sighash = liftup
                                    .sighash()
                                    .map_err(BatchExecutionError::LiftupSighashError)?;
                                executed_entry_sighashes.push(sighash);
                            }

                            // 12.3.a.1.a.3 Add the BLS key of the `RootAccount` of the `Liftup`.
                            {
                                let account_bls_key = liftup.root_account.bls_key();
                                executed_entry_account_bls_keys.push(account_bls_key);
                            }
                        }
                        // 12.3.a.1.b Error.
                        Err(error) => return Err(BatchExecutionError::LiftupExecutionError(error)),
                    }
                }
                _ => panic!("Not implemented yet."),
            }

            // 12.1 Break out of the loop if the APE bitstream is empty.
            if ape_bitstream.next().is_none() {
                break;
            }
        }

        // 13 Verify the aggregate BLS signature.
        if !bls_verify_aggregate(
            executed_entry_account_bls_keys,
            executed_entry_sighashes,
            aggregate_bls_signature,
        ) {
            return Err(BatchExecutionError::AggregateBLSSignatureVerificationError);
        }

        // 14 Return the batch height, payload version, batch timestamp, aggregate BLS signature, and executed entries.
        Ok((
            batch_height,
            payload_version,
            batch_timestamp,
            aggregate_bls_signature,
            executed_entries,
        ))
    }

    /// Executes a `Liftup` Entry.
    pub async fn execute_liftup(
        &mut self,
        liftup: &Liftup,
        execution_timestamp: u64,
    ) -> Result<Entry, LiftupExecutionError> {
        // 1 Execute the liftup.
        match self
            .execute_liftup_internal(liftup, execution_timestamp)
            .await
        {
            // 1.a Success.
            Ok(_) => {
                // 1.a.1 Construct the Liftup entry.
                let liftup_entry = Entry::new_liftup(liftup.clone());

                // 1.a.2 Return the liftup entry.
                Ok(liftup_entry)
            }
            // 1.b Error.
            Err(error) => Err(error),
        }
    }
}
