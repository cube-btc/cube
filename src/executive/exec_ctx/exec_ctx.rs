use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use crate::constructive::core_types::valtypes::val::long_val::long_val::LongVal;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;
use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::txn::ext::OutpointExt;
use crate::constructive::txout_types::payload::payload::Payload;
use crate::constructive::txout_types::projector::projector::Projector;
use crate::executive::exec_ctx::errors::batch_execution_error::BatchExecutionError;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::transmutative::bls::verify::bls_verify_aggregate;
use crate::transmutative::codec::bitvec_ext::BitVecExt;
use crate::{
    constructive::entry::entry_kinds::liftup::liftup::Liftup,
    executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError,
    executive::exec_ctx::errors::apply_changes_error::ApplyChangesError,
};
use bit_vec::BitVec;
use bitcoin::hashes::Hash;
use bitcoin::OutPoint;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A type alias for the batch height.
type NewBatchHeight = u64;

/// A type alias for the payload version.
type PayloadVersion = u32;

/// A type alias for the batch timestamp.
type BatchTimestamp = u64;

/// A type alias for the aggregate BLS signature.
type AggregateBLSSignature = [u8; 96];

/// A type alias for the expired projector outpoints.
type ExpiredProjectorOutpoints = Vec<OutPoint>;

/// A type alias for the new payload.
type NewPayload = Payload;

/// A type alias for the new projector.
type NewProjector = Option<Projector>;

/// A type alias for the batch txid.
type BatchTxid = [u8; 32];

/// `ExecCtx` contains a set of executed entries.
pub struct ExecCtx {
    // The key of the Engine.
    pub engine_key: [u8; 32],

    // The local sync manager database of the Engine.
    pub sync_manager: SYNC_MANAGER,

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
        sync_manager: SYNC_MANAGER,
        utxo_set: UTXO_SET,
        registery: REGISTERY,
        graveyard: GRAVEYARD,
        coin_manager: COIN_MANAGER,
        flame_manager: FLAME_MANAGER,
    ) -> EXEC_CTX {
        // 1 Initialize the `ExecCtx`.
        let exec_ctx = ExecCtx {
            engine_key,
            sync_manager,
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
        {
            self.coin_manager.lock().await.pre_execution();
        }
        // 2 Pre-execution graveyard.
        {
            self.graveyard.lock().await.pre_execution();
        }

        // 3 Pre-execution registery.
        {
            self.registery.lock().await.pre_execution();
        }

        // 4 Pre-execution flame manager.
        {
            self.flame_manager.lock().await.pre_execution();
        }
    }

    /// Rolls back the last execution of the `ExecCtx` due to a failed individual Entry execution.
    pub async fn rollback_last(&mut self) {
        // 1 Rollback last coin manager.
        {
            self.coin_manager.lock().await.rollback_last();
        }

        // 2 Rollback last graveyard.
        {
            self.graveyard.lock().await.rollback_last();
        }

        // 3 Rollback last registery.
        {
            self.registery.lock().await.rollback_last();
        }

        // 4 Rollback last flame manager.
        {
            self.flame_manager.lock().await.rollback_last();
        }
    }

    /// Flushes all the changes in the `ExecCtx`.
    pub async fn flush(&mut self) {
        // 1 Rollback all coin manager.
        {
            self.coin_manager.lock().await.flush_delta();
        }

        // 2 Rollback all graveyard.
        {
            self.graveyard.lock().await.flush_deltas();
        }

        // 3 Rollback all registery.
        {
            self.registery.lock().await.flush_delta();
        }

        // 4 Rollback all flame manager.
        {
            self.flame_manager.lock().await.flush_delta();
        }
    }

    /// Applies the changes to the `ExecCtx` collectively for all Entries in the container.
    pub async fn apply_changes(
        &mut self,
        new_batch_height: u64,
        new_payload: Payload,
        spent_bitcoin_tx_inputs: Vec<OutPoint>,
        projector_expiry_gap: u64,
    ) -> Result<(), ApplyChangesError> {
        // 1 Calculate the projector expiry height.
        let projector_expiry_height = new_batch_height + projector_expiry_gap;

        // 2 Apply changes to the coin manager.
        {
            // 2.1 Lock the coin manager.
            let mut _coin_manager = self.coin_manager.lock().await;

            // 2.2 Apply changes to the coin manager.
            if let Err(error) = _coin_manager.apply_changes() {
                return Err(ApplyChangesError::CoinManagerApplyChangesError(error));
            }
        }

        // 3 Apply changes to the graveyard.
        {
            // 3.1 Lock the graveyard.
            let mut _graveyard = self.graveyard.lock().await;

            // 3.2 Apply changes to the graveyard.
            if let Err(error) = _graveyard.apply_changes() {
                return Err(ApplyChangesError::GraveyardApplyChangesError(error));
            }
        }

        // 4 Apply changes to the registery.
        {
            // 4.1 Lock the registery.
            let mut _registery = self.registery.lock().await;

            // 4.2 Apply changes to the registery.
            if let Err(error) = _registery.apply_changes() {
                return Err(ApplyChangesError::RegisteryApplyChangesError(error));
            }
        }

        // 5 Apply changes to the flame manager.
        {
            // 5.1 Lock the flame manager.
            let mut _flame_manager = self.flame_manager.lock().await;

            // 5.2 Apply changes to the flame manager.
            if let Err(error) = _flame_manager
                .apply_changes(
                    &self.coin_manager,
                    &self.registery,
                    new_batch_height,
                    projector_expiry_height,
                )
                .await
            {
                return Err(ApplyChangesError::FlameManagerApplyChangesError(error));
            }
        }

        // 6 Update tips in the sync manager.
        {
            // 6.1 Lock the sync manager.
            let mut _sync_manager = self.sync_manager.lock().await;

            // 6.2 Update the cube batch sync height tip.
            _sync_manager.set_cube_batch_sync_height_tip(new_batch_height);

            // 6.3 Update the payload tip.
            _sync_manager.set_payload_tip(new_payload);
        }

        // 7 Safe-remove spent lift tx inputs from the utxo set (as this may be in-flight execution).
        {
            // 7.1 Lock the utxo set.
            let mut _utxo_set = self.utxo_set.lock().await;

            // 7.2 Safe-remove spent lift tx inputs from the utxo set.
            _utxo_set.safe_remove_utxos(spent_bitcoin_tx_inputs);
        }

        // 8 Flush the container changes.
        self.flush().await;

        // 9 Return Ok.
        Ok(())
    }

    /// Executes a batch.
    pub async fn execute_batch(
        &mut self,
        batch_container: BatchContainer,
    ) -> Result<
        (
            NewBatchHeight,
            BatchTxid,
            PayloadVersion,
            BatchTimestamp,
            AggregateBLSSignature,
            Vec<Entry>,
            ExpiredProjectorOutpoints,
            NewPayload,
            NewProjector,
        ),
        BatchExecutionError,
    > {
        // 1 Get the batch height.
        let new_batch_height = batch_container.batch_height();

        // 2 Check if the new batch height is valid.
        {
            // 2.1 Get the current batch sync height tip.
            let current_batch_sync_height_tip = {
                // 2.1.1 Lock the sync manager.
                let _sync_manager = self.sync_manager.lock().await;

                // 2.1.2 Get the current batch sync height tip.
                _sync_manager.cube_batch_sync_height_tip()
            };

            // 2.2 Check if the new batch height is one plus of the current batch sync height tip.
            if new_batch_height != current_batch_sync_height_tip + 1 {
                return Err(BatchExecutionError::InvalidNewBatchHeightError(
                    current_batch_sync_height_tip,
                    new_batch_height,
                ));
            }
        }

        // 3 Get the payload bytes.
        let payload_bytes = batch_container.payload_bytes();

        // 4 Get the Bitcoin transaction inputs.
        let bitcoin_tx_inputs = batch_container.bitcoin_tx_inputs();

        // 5 Get the Bitcoin transaction outputs.
        let bitcoin_tx_outputs = batch_container.bitcoin_tx_outputs();

        // 6 Convert the payload bytes to APE bits.
        let payload_ape_bits = BitVec::from_ape_payload_bytes(payload_bytes.clone()).ok_or(
            BatchExecutionError::BatchTemplatePayloadBitsConversionError(payload_bytes.clone()),
        )?;

        // 7 Turn the APE bits into an iterator.
        let mut ape_bitstream = payload_ape_bits.iter();

        // 8 Turn the Bitcoin transaction inputs into an iterator.
        let mut bitcoin_tx_inputs_iter = bitcoin_tx_inputs.into_iter();

        // 9 Turn the Bitcoin transaction outputs into an iterator.
        let mut bitcoin_tx_outputs_iter = bitcoin_tx_outputs.iter();

        // 10 Return params from the params manager: TODO
        let decode_account_rank_as_longval = true;
        let decode_contract_rank_as_longval = true;
        let base_ops_price = 100;

        // 11 Decode payload version as as shortcal and session timestamp as a longval.
        let payload_version: u32 = ShortVal::decode_ape(&mut ape_bitstream)
            .map_err(BatchExecutionError::DecodePayloadVersionError)?
            .value();

        // 12 Decode batch timestamp as a longval used as the execution timestamp for all entries.
        let batch_timestamp: u64 = LongVal::decode_ape(&mut ape_bitstream)
            .map_err(BatchExecutionError::DecodeBatchTimestampError)?
            .value();

        // 13 Decode aggregate BLS signature as a byte array.
        let aggregate_bls_signature: [u8; 96] = {
            // 13.1 Collect 768 bits from the APE bitstream.
            let aggregate_bls_signature_bits: BitVec = ape_bitstream.by_ref().take(768).collect();
            if aggregate_bls_signature_bits.len() != 768 {
                return Err(BatchExecutionError::DecodeAggregateBLSSignatureError);
            }

            // 13.2 Convert the aggregate BLS signature bits to a byte array.
            let bytes = aggregate_bls_signature_bits.to_bytes();
            bytes
                .try_into()
                .map_err(|_| BatchExecutionError::DecodeAggregateBLSSignatureError)?
        };

        // 14 Decode the expired projectors count as a shortval.
        let expired_projectors_count: u32 = ShortVal::decode_ape(&mut ape_bitstream)
            .map_err(BatchExecutionError::DecodeExpiredProjectorsCountError)?
            .value();

        // 15 Collect the projector presence bit.
        let projector_presence_bit: bool = ape_bitstream
            .next()
            .ok_or(BatchExecutionError::FailedToCollectProjectorPresenceBitError)?;

        // 16 Iterate one tx input to get the prev payload outpoint.
        let prev_payload_outpoint = {
            // 16.1 Iterate one tx input for the payload.
            let bitcoin_tx_input = bitcoin_tx_inputs_iter
                .next()
                .ok_or(BatchExecutionError::FailedToIterAndGetPayloadTxInputError)?;

            // 16.2 Return the payload outpoint.
            bitcoin_tx_input.clone()
        };

        // 17 Check if the prev payload outpoint matches to the payload tip outpoint in the sync manager.
        {
            // 17.2 Get the current tx id tip.
            let payload_tip = {
                // 17.2.1 Lock the sync manager.
                let _sync_manager = self.sync_manager.lock().await;

                // 17.2.2 Get the current payload tip.
                _sync_manager.payload_tip()
            };

            // 17.3 Get the payload tip outpoint.
            let payload_tip_outpoint = payload_tip
                .outpoint()
                .ok_or(BatchExecutionError::PayloadTipLocationNotFoundError)?;

            // 17.4 Check if the prev payload outpoint matches to the payload tip outpoint.
            if prev_payload_outpoint != payload_tip_outpoint {
                return Err(BatchExecutionError::PayloadOutpointMismatchError(
                    payload_tip_outpoint,
                    prev_payload_outpoint,
                ));
            }
        }

        // 18 Iterate one tx output and get the new payload tx output.
        let new_payload_txout = {
            let bitcoin_tx_output = bitcoin_tx_outputs_iter
                .next()
                .ok_or(BatchExecutionError::FailedToIterAndGetPayloadTxOutputError)?;

            bitcoin_tx_output.clone()
        };

        // 19 Construct the new payload.
        let new_payload: Payload = {
            // 19.1 Get the new payload outpoint.
            let new_payload_outpoint =
                OutPoint::from_txid_and_vout(batch_container.signed_batch_txn.txid(), 0);

            // 19.2 Construct the new payload.
            Payload::new(
                self.engine_key,
                payload_bytes.clone(),
                Some((new_payload_outpoint, new_payload_txout)),
            )
        };

        // 20 Initialize the expired projector outpoints list.
        let mut expired_projector_outpoints: Vec<OutPoint> = Vec::new();

        // 21 Iterate expired projectors: Placeholder for the time being.
        // Expired projectors logic is not supported for the time being.
        {
            for _ in 0..expired_projectors_count {
                // 21.1 Iterate one tx input for the expired projector.
                let expired_projector_outpoint = bitcoin_tx_inputs_iter
                    .next()
                    .ok_or(BatchExecutionError::FailedToIterateExpiredProjectorsError)?
                    .clone();

                // 21.2 Add the expired projector outpoint to the expired projector outpoints list.
                expired_projector_outpoints.push(expired_projector_outpoint);
            }
        }

        // 22 If projector is present, iterate one output to get the projector tx output.
        // Not yet supported for the time being.
        let new_projector: Option<Projector> = {
            match projector_presence_bit {
                // 22.a The projector is present.
                true => {
                    // 22.a.1 Iterate one tx output for the projector.
                    let bitcoin_tx_output = bitcoin_tx_outputs_iter
                        .next()
                        .ok_or(BatchExecutionError::FailedToIterAndGetProjectorTxOutputError)?;

                    // 22.a.2 Get the new projector outpoint.
                    let new_projector_outpoint =
                        OutPoint::from_txid_and_vout(batch_container.signed_batch_txn.txid(), 1);

                    // 22.a.3 Construct the projector.
                    let projector = Some(Projector {
                        scriptpubkey: bitcoin_tx_output.script_pubkey.as_bytes().to_vec(),
                        satoshi_amount: bitcoin_tx_output.value.to_sat(),
                        location: Some((new_projector_outpoint, bitcoin_tx_output.clone())),
                    });

                    // 22.a.4 Return the projector.
                    projector
                }

                // 22.b The projector is not present.
                false => None,
            }
        };

        // 23 Initialize the executed entries list to collect the executed entries.
        let mut executed_entries: Vec<Entry> = Vec::new();

        // 24 Initialize the executed entry sighashes list.
        let mut executed_entry_sighashes: Vec<[u8; 32]> = Vec::new();

        // 25 Initialize the executed entry account BLS keys list.
        let mut executed_entry_account_bls_keys: Vec<[u8; 48]> = Vec::new();

        // 26 Decode entries from the patload one by one and execute them.
        loop {
            // 26.1 Decode Entry from the APE bitstream.
            let entry = Entry::decode_ape(
                self.engine_key,
                new_batch_height,
                &mut ape_bitstream,
                &mut bitcoin_tx_inputs_iter,
                decode_account_rank_as_longval,
                decode_contract_rank_as_longval,
                base_ops_price,
                &self.utxo_set,
                &self.registery,
            )
            .await
            .map_err(BatchExecutionError::DecodeEntryError)?;

            // 26.2 Execute the decoded `Entry`.
            match entry {
                // 26.2.a The `Entry` is a `Liftup`.
                Entry::Liftup(liftup) => {
                    // 26.2.a.1 Execute the `Liftup` `Entry`.
                    match self.execute_liftup(&liftup, batch_timestamp).await {
                        // 26.2.a.1.a Success.
                        Ok(entry) => {
                            // 26.2.a.1.a.1 Add the liftup entry to the executed entries.
                            executed_entries.push(entry);

                            // 26.2.a.1.a.2 Add the sighash of the `Liftup`.
                            {
                                let sighash = liftup
                                    .sighash()
                                    .map_err(BatchExecutionError::LiftupSighashError)?;
                                executed_entry_sighashes.push(sighash);
                            }

                            // 26.2.a.1.a.3 Add the BLS key of the `RootAccount` of the `Liftup`.
                            {
                                let account_bls_key = liftup.root_account.bls_key();
                                executed_entry_account_bls_keys.push(account_bls_key);
                            }
                        }
                        // 26.2.a.1.b Error.
                        Err(error) => return Err(BatchExecutionError::LiftupExecutionError(error)),
                    }
                }
                _ => panic!("Not implemented yet."),
            }

            // 26.1 Break out of the loop if the APE bitstream is empty.
            if ape_bitstream.next().is_none() {
                break;
            }
        }

        // 27 Verify the aggregate BLS signature.
        if !bls_verify_aggregate(
            executed_entry_account_bls_keys,
            executed_entry_sighashes,
            aggregate_bls_signature,
        ) {
            return Err(BatchExecutionError::AggregateBLSSignatureVerificationError);
        }

        // 28 Get the batch txid.
        let batch_txid: [u8; 32] = batch_container.signed_batch_txn.txid().to_byte_array();

        // 29 Return the batch height, payload version, batch timestamp, aggregate BLS signature, executed entries, and the new payload.
        Ok((
            new_batch_height,
            batch_txid,
            payload_version,
            batch_timestamp,
            aggregate_bls_signature,
            executed_entries.clone(),
            expired_projector_outpoints.clone(),
            new_payload.clone(),
            new_projector.clone(),
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
