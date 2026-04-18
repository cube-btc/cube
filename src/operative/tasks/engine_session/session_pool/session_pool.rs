use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use crate::constructive::bitcoiny::batch_txn::signed_batch_txn::signed_batch_txn::SignedBatchTxn;
use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::txout_types::payload::payload::Payload;
use crate::constructive::txout_types::projector::projector::Projector;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::executive::exec_ctx::exec_ctx::EXEC_CTX;
use crate::inscriptive::archival_manager::archival_manager::ARCHIVAL_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::state_manager::state_manager::STATE_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::tasks::engine_session::session_pool::error::exec_liftup_in_pool_error::ExecLiftupInPoolError;
use crate::operative::tasks::engine_session::session_pool::error::into_batch_container_error::IntoBatchContainerError;
use crate::transmutative::bls::agg::bls_aggregate;
use crate::transmutative::codec::bitvec_ext::BitVecExt;
use crate::transmutative::key::KeyHolder;
use bit_vec::BitVec;
use bitcoin::{OutPoint, TxOut};
use bls_on_arkworks::errors::BLSError;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

/// The maximum number of entries that can be in the pool.
const MAX_IN_POOL_ENTRIES: usize = 1000;

/// The state of the `SessionPool`.
pub enum SessionPoolState {
    // The session pool is inactive.
    Inactive,
    // The session pool is active.
    Active,
    // The pool is suspended for some reason.
    Suspended,
    // The pool is overloaded.
    Overloaded,
}

/// `SessionPool` represents the local mempool of the Engine, containing a collection of entries that have been locally executed and pooled.
pub struct SessionPool {
    // The state of the session pool.
    pub state: SessionPoolState,

    // The engine key.
    pub engine_key: [u8; 32],

    pub sync_manager: SYNC_MANAGER,

    // The utxo set.
    pub utxo_set: UTXO_SET,

    // The registery.
    pub registery: REGISTERY,

    // The graveyard.
    pub graveyard: GRAVEYARD,

    // The coin manager.
    pub coin_manager: COIN_MANAGER,

    // The flame manager.
    pub flame_manager: FLAME_MANAGER,

    // The state manager.
    pub state_manager: STATE_MANAGER,

    // The exec container.
    pub exec_container: EXEC_CTX,

    // The entries that have been added in the pool.
    pub added_entries: Vec<Entry>,

    // The Bitcoin transaction inputs that have been added.
    pub added_tx_inputs: Vec<(OutPoint, TxOut)>,

    // The Bitcoin transaction outputs that have been added.
    pub added_tx_outputs: Vec<TxOut>,

    // The individual `Entry` BLS signatures that have been added.
    pub added_individual_entry_bls_signatures: Vec<[u8; 96]>,
}

/// Guarded `SessionPool`.
#[allow(non_camel_case_types)]
pub type SESSION_POOL = Arc<Mutex<SessionPool>>;

impl SessionPool {
    /// Constructs the `SessionPool`.    
    pub fn construct(
        engine_key: [u8; 32],
        sync_manager: &SYNC_MANAGER,
        utxo_set: &UTXO_SET,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
        coin_manager: &COIN_MANAGER,
        flame_manager: &FLAME_MANAGER,
        state_manager: &STATE_MANAGER,
        archival_manager: Option<ARCHIVAL_MANAGER>,
    ) -> SESSION_POOL {
        // 1 Construct the exec container.
        let exec_container = ExecCtx::construct(
            engine_key,
            Arc::clone(sync_manager),
            Arc::clone(utxo_set),
            Arc::clone(registery),
            Arc::clone(graveyard),
            Arc::clone(coin_manager),
            Arc::clone(flame_manager),
            Arc::clone(state_manager),
            archival_manager,
        );

        // 2 Construct the session pool.
        let session_pool = SessionPool {
            state: SessionPoolState::Inactive,
            engine_key,
            sync_manager: Arc::clone(sync_manager),
            utxo_set: Arc::clone(utxo_set),
            registery: Arc::clone(registery),
            graveyard: Arc::clone(graveyard),
            coin_manager: Arc::clone(coin_manager),
            flame_manager: Arc::clone(flame_manager),
            state_manager: Arc::clone(state_manager),
            exec_container,
            added_entries: Vec::new(),
            added_tx_inputs: Vec::new(),
            added_tx_outputs: Vec::new(),
            added_individual_entry_bls_signatures: Vec::new(),
        };

        // 3 Guard the session pool.
        let guarded_session_pool: SESSION_POOL = Arc::new(Mutex::new(session_pool));

        // 4 Return the guarded session pool.
        guarded_session_pool
    }

    /// Flushes the `SessionPool`.
    async fn flush(&mut self) {
        // 1 Flush the execution context.
        {
            // 1.1 Lock the execution context.
            let mut _exec_container = self.exec_container.lock().await;

            // 1.2 Flush the execution context.
            _exec_container.flush().await;
        }

        // 2 Reset the added entries.
        self.added_entries = Vec::new();

        // 3 Reset the APE encoded entries.
        self.added_tx_inputs = Vec::new();

        // 4 Reset the added Bitcoin transaction outputs.
        self.added_tx_outputs = Vec::new();

        // 6 Reset the added individual entry BLS signatures.
        self.added_individual_entry_bls_signatures = Vec::new();
    }

    /// Starts the session of the `SessionPool`.
    pub fn begin_session(&mut self) {
        self.state = SessionPoolState::Active;
    }

    /// Suspends the session of the `SessionPool`.
    pub fn suspend_session(&mut self) {
        self.state = SessionPoolState::Suspended;
    }

    /// Resumes the session of the `SessionPool`.
    pub fn resume_session(&mut self) {
        self.state = SessionPoolState::Active;
    }

    /// Ends the session of the `SessionPool`.
    pub async fn end_session(&mut self) {
        // 1 Set the state of the session pool to inactive.
        self.state = SessionPoolState::Inactive;

        // 2 Flush the session pool.
        self.flush().await;
    }

    /// Aggregates the BLS signatures of the added entries.
    pub fn aggregate_bls_signature(&self) -> Result<[u8; 96], BLSError> {
        bls_aggregate(self.added_individual_entry_bls_signatures.clone())
    }

    /// Converts the `ExecCtx` into a `BatchContainer`.
    pub async fn into_batch_container(
        &mut self,
        batch_timestamp: u64,
        payload_version: u32,
        bitcoin_transaction_fee: u64,
        engine_keyholder: &KeyHolder,
    ) -> Result<BatchContainer, IntoBatchContainerError> {
        // 1 Retrieve tips from the sync manager.
        let (new_cube_batch_height, prev_payload): (u64, Payload) = {
            // 1.1 Lock the sync manager.
            let _sync_manager = self.sync_manager.lock().await;

            // 1.2 Get the new cube batch height.
            let new_cube_batch_height = _sync_manager.cube_batch_sync_height_tip() + 1;

            // 1.3 Get the prev payload.
            let prev_payload = _sync_manager.payload_tip();

            // 1.4 Return the new cube batch height and prev payload.
            (new_cube_batch_height, prev_payload)
        };

        // 2 Initialize the bit vector for the payload.
        let mut payload_bits: BitVec = BitVec::new();

        // 3 Encode the payload version.
        {
            // 3.1 Encode the payload version.
            let payload_version_bits = ShortVal::new(payload_version).encode_ape();

            // 3.2 Extend the payload bits with the payload version bits.
            payload_bits.extend(payload_version_bits);
        }

        // 4 Encode the batch timestamp.
        {
            // 4.1 Encode the batch timestamp.
            let batch_timestamp_bits = LongVal::new(batch_timestamp).encode_ape();

            // 4.2 Extend the payload bits with the batch timestamp bits.
            payload_bits.extend(batch_timestamp_bits);
        }

        // 5 Encode the aggregate BLS signature.
        {
            // 5.1 Get the aggregate BLS signature.
            let aggregate_bls_signature = self
                .aggregate_bls_signature()
                .map_err(|_| IntoBatchContainerError::AggregateBLSSignatureError)?;

            // 5.2 Convert the aggregate BLS signature to bits.
            let aggregate_bls_signature_bits = BitVec::from_bytes(&aggregate_bls_signature);

            // 5.3 Extend the payload bits with the aggregate BLS signature bits.
            payload_bits.extend(aggregate_bls_signature_bits);
        }

        // 6 Encode the expired projectors count: Currently always encoded as 0 as a placeholder.
        // Expired projectors logic is not supported for the time being.
        {
            // 6.1 Set the expired projectors count to 0 as a placeholder.
            let expired_projectors_count = 0;

            // 6.2 Encode the expired projectors count.
            let expired_projectors_count_bits =
                ShortVal::new(expired_projectors_count).encode_ape();

            // 6.3 Extend the payload bits with the expired projectors count bits.
            payload_bits.extend(expired_projectors_count_bits);
        }

        // 7 Retrieve the new projector.
        // Not set for the time being.
        let new_projector: Option<Projector> = None;

        // 8 Insert a bit to the beginning of the payload bits to indicate the presence of the new projector.
        match new_projector {
            // 8.a The new projector is set.
            Some(_) => {
                // 8.a.1 Push true bit to indicate the presence of the projector.
                payload_bits.push(true);
            }
            // 8.b The new projector is not set.
            None => {
                // 8.b.1 Push false bit to indicate the absence of the projector.
                payload_bits.push(false);
            }
        }

        // 9 Encode the added entries.
        for entry in &self.added_entries {
            // 9.1 Encode the entry as APE bits.
            let entry_ape_bits = entry
                .encode_ape(new_cube_batch_height, &self.registery, true, true)
                .await
                .map_err(IntoBatchContainerError::EntryAPEEncodeError)?;

            // 9.2 Extend the payload bits with the entry APE bits.
            payload_bits.extend(entry_ape_bits);
        }

        // 10 Convert the payload bits to payload bytes.
        let new_payload_bytes: Bytes = payload_bits.to_ape_payload_bytes();

        // 11 Get prev projectors from sync manager: Not implemented for the time being.
        let prev_projectors = Vec::<Projector>::new();

        // 12 Get the executed entries.
        let executed_entries: Vec<Entry> = self.added_entries.clone();

        // 13 Construct the new payload.
        let new_payload = Payload::new(self.engine_key, new_payload_bytes.clone(), None);

        // 14 Construct the signed batch transaction.
        let signed_batch_txn = SignedBatchTxn::construct(
            prev_payload,
            prev_projectors,
            executed_entries,
            new_payload,
            new_projector,
            bitcoin_transaction_fee,
            engine_keyholder,
        )
        .map_err(|err| IntoBatchContainerError::SignedBatchTxnConstructError(err))?;

        // 15 Construct the batch container.
        let batch_container = BatchContainer::new(
            new_cube_batch_height,
            new_payload_bytes,
            signed_batch_txn,
        );

        // 16 Return the batch container.
        Ok(batch_container)
    }

    /// Executes a `Liftup` entry in the `SessionPool`.
    pub async fn exec_liftup_in_pool(
        &mut self,
        execution_timestamp: u64,
        liftup: &Liftup,
        liftup_bls_signature: [u8; 96],
    ) -> Result<Entry, ExecLiftupInPoolError> {
        // 1 Check the pool session status.
        match self.state {
            // 1.a The session is inactive.
            SessionPoolState::Inactive => {
                return Err(ExecLiftupInPoolError::SessionInactiveError);
            }
            // 1.b The session is suspended.
            SessionPoolState::Suspended => {
                return Err(ExecLiftupInPoolError::SessionSuspendedError);
            }
            // 1.c The session is active but it might be overloaded.
            _ => {
                // 1.c.1 Check if the pool is overloaded.
                if self.added_entries.len() >= MAX_IN_POOL_ENTRIES {
                    return Err(ExecLiftupInPoolError::PoolOverloadedError);
                }
            }
        };

        // 2 Retrieve tips from the sync manager.
        let new_cube_batch_height: u64 = {
            // 2.1 Lock the sync manager.
            let _sync_manager = self.sync_manager.lock().await;

            // 2.2 Get the new cube batch height.
            let new_cube_batch_height = _sync_manager.cube_batch_sync_height_tip() + 1;

            // 2.3 Return the new cube batch height.
            new_cube_batch_height
        };

        // 3 Run pre-validations.
        {
            liftup
                .validate_overall(
                    self.engine_key,
                    new_cube_batch_height,
                    &self.utxo_set,
                    &self.registery,
                    &self.graveyard,
                    liftup_bls_signature,
                )
                .await
                .map_err(|err| ExecLiftupInPoolError::LiftupValidateOverallError(err))?;
        }

        // 4 Prepare for the execution by backing up the execution context.
        {
            let mut _exec_container = self.exec_container.lock().await;
            _exec_container.pre_execution().await;
        }

        // 5 Execute the liftup in the execution context.
        match self
            .exec_container
            .lock()
            .await
            .execute_liftup(liftup, execution_timestamp)
            .await
        {
            // 5.a Success.
            Ok(liftup_entry) => {
                // 5.a.1 Add the liftup entry to the added entries.
                self.added_entries.push(liftup_entry.clone());

                // 5.a.2 Add the liftup BLS signature to the added individual entry BLS signatures.
                self.added_individual_entry_bls_signatures
                    .push(liftup_bls_signature);

                // 5.a.3 Add internal Lifts inside the Liftup to the added Bitcoin transaction inputs.
                {
                    self.added_tx_inputs.extend(
                        liftup
                            .lift_tx_inputs
                            .iter()
                            .map(|lift| (lift.outpoint(), lift.txout().clone())),
                    );
                }

                // 5.a.4 Return the liftup entry.
                Ok(liftup_entry)
            }

            // 5.b Error.
            Err(error) => {
                // 5.b.1 Rollback the execution.
                {
                    self.exec_container.lock().await.rollback_last().await;
                }

                // 5.b.2 Return the error.
                Err(ExecLiftupInPoolError::LiftupExecutionError(error))
            }
        }
    }
}
