use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use crate::constructive::bitcoiny::batch_txn::signed_batch_txn::signed_batch_txn::SignedBatchTxn;
use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
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
use crate::inscriptive::privileges_manager::privileges_manager::PRIVILEGES_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::state_manager::state_manager::STATE_MANAGER;
use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::tasks::engine_session::session_pool::error::exec_liftup_in_pool_error::ExecLiftupInPoolError;
use crate::operative::tasks::engine_session::session_pool::error::exec_move_in_pool_error::ExecMoveInPoolError;
use crate::operative::tasks::engine_session::session_pool::error::into_batch_container_error::IntoBatchContainerError;
use crate::transmutative::bls::agg::bls_aggregate;
use crate::transmutative::codec::bitvec_ext::BitVecExt;
use crate::transmutative::key::KeyHolder;
use bit_vec::BitVec;
use bls_on_arkworks::errors::BLSError;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A type alias for the batch height.
pub type BatchHeight = u64;

/// A type alias for the batch timestamp.
pub type BatchTimestamp = u64;

/// A type alias for a pooled entry identifier (32-byte hash).
pub type EntryId = [u8; 32];

/// A type alias for the Bitcoin transaction fee.
type BitcoinTransactionFee = u64;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

/// The payload version is currently hardcoded to 1.
const PAYLOAD_VERSION: u32 = 1;

/// The maximum number of entries that can be in the pool.
const MAX_IN_POOL_ENTRIES: usize = 1000;

/// The state of the `SessionPool`.
pub enum SessionPoolState {
    // The session pool is inactive.
    Inactive,
    // The session pool is active.
    Active,
    // The session pool has taken a break.
    Break,
    // The pool is suspended for some reason.
    Suspended,
    // The pool is overloaded.
    Overloaded,
}

/// `SessionPool` represents the local mempool of the Engine, containing a collection of entries that have been locally executed and pooled.
pub struct SessionPool {
    // The state of the session pool.
    pub state: SessionPoolState,

    // The batch height.
    pub batch_info: Option<(BatchHeight, BatchTimestamp, BitcoinTransactionFee)>,

    // The engine key.
    pub engine_key: [u8; 32],

    // The sync manager.
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

    // The privileges manager.
    pub privileges_manager: PRIVILEGES_MANAGER,

    // The exec context.
    pub exec_ctx: EXEC_CTX,

    // The entries that have been added in the pool.
    pub added_entries: Vec<Entry>,

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
        privileges_manager: &PRIVILEGES_MANAGER,
        archival_manager: Option<ARCHIVAL_MANAGER>,
    ) -> SESSION_POOL {
        // 1 Construct the exec context.
        let exec_ctx = ExecCtx::construct(
            engine_key,
            Arc::clone(sync_manager),
            Arc::clone(utxo_set),
            Arc::clone(registery),
            Arc::clone(graveyard),
            Arc::clone(coin_manager),
            Arc::clone(flame_manager),
            Arc::clone(state_manager),
            Arc::clone(privileges_manager),
            archival_manager,
        );

        // 2 Construct the session pool.
        let session_pool = SessionPool {
            state: SessionPoolState::Inactive,
            batch_info: None,
            engine_key,
            sync_manager: Arc::clone(sync_manager),
            utxo_set: Arc::clone(utxo_set),
            registery: Arc::clone(registery),
            graveyard: Arc::clone(graveyard),
            coin_manager: Arc::clone(coin_manager),
            flame_manager: Arc::clone(flame_manager),
            state_manager: Arc::clone(state_manager),
            privileges_manager: Arc::clone(privileges_manager),
            exec_ctx,
            added_entries: Vec::new(),
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
            let mut _exec_ctx = self.exec_ctx.lock().await;

            // 1.2 Flush the execution context.
            _exec_ctx.flush().await;
        }

        // 2 Reset the added entries.
        self.added_entries = Vec::new();

        // 3 Reset the added individual entry BLS signatures.
        self.added_individual_entry_bls_signatures = Vec::new();

        // 4 Reset the batch height.
        self.batch_info = None;
    }

    /// Starts the session of the `SessionPool`.
    ///
    /// The new-account registration tracker is cleared when the pool flushes the execution context
    /// (`end_session` → `flush` → `ExecCtx::flush`).
    pub fn begin_session(
        &mut self,
        batch_height: u64,
        batch_timestamp: u64,
        bitcoin_transaction_fee: u64,
    ) {
        // 1 Set the state of the session pool to active.
        self.state = SessionPoolState::Active;

        // 2 Set the batch info.
        self.batch_info = Some((batch_height, batch_timestamp, bitcoin_transaction_fee));
    }

    /// Suspends the session of the `SessionPool`.
    pub fn suspend_session(&mut self) {
        self.state = SessionPoolState::Suspended;
    }

    /// Takes a break.
    pub fn take_a_break_session(&mut self) {
        self.state = SessionPoolState::Break;
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
        engine_keyholder: &KeyHolder,
    ) -> Result<BatchContainer, IntoBatchContainerError> {
        // 1 Get the batch info.
        let (batch_height, batch_timestamp, bitcoin_transaction_fee) = self
            .batch_info
            .ok_or(IntoBatchContainerError::BatchInfoNotFoundError)?;

        // 2 Get params from the params manager: Placeholder for now.
        let (encode_account_rank_as_longval, encode_contract_rank_as_longval) = (false, false);

        // 3 Retrieve the prev payload.
        let prev_payload = {
            // 3.1 Lock the sync manager.
            let _sync_manager = self.sync_manager.lock().await;

            // 3.2 Get the prev payload.
            _sync_manager.payload_tip()
        };

        // 4 Initialize the bit vector for the payload.
        let mut payload_bits: BitVec = BitVec::new();

        // 5 Encode the payload version.
        {
            // 5.1 Encode the payload version.
            let payload_version_bits = ShortVal::new(PAYLOAD_VERSION).encode_ape();

            // 5.2 Extend the payload bits with the payload version bits.
            payload_bits.extend(payload_version_bits);
        }

        // 6 Encode the batch timestamp.
        {
            // 6.1 Encode the batch timestamp.
            let batch_timestamp_bits = LongVal::new(batch_timestamp).encode_ape();

            // 6.2 Extend the payload bits with the batch timestamp bits.
            payload_bits.extend(batch_timestamp_bits);
        }

        // 7 Encode the aggregate BLS signature.
        {
            // 7.1 Get the aggregate BLS signature.
            let aggregate_bls_signature = self
                .aggregate_bls_signature()
                .map_err(|_| IntoBatchContainerError::AggregateBLSSignatureError)?;

            // 7.2 Convert the aggregate BLS signature to bits.
            let aggregate_bls_signature_bits = BitVec::from_bytes(&aggregate_bls_signature);

            // 7.3 Extend the payload bits with the aggregate BLS signature bits.
            payload_bits.extend(aggregate_bls_signature_bits);
        }

        // 8 Encode the expired projectors count: Currently always encoded as 0 as a placeholder.
        // Expired projectors logic is not supported for the time being.
        {
            // 8.1 Set the expired projectors count to 0 as a placeholder.
            let expired_projectors_count = 0;

            // 8.2 Encode the expired projectors count.
            let expired_projectors_count_bits =
                ShortVal::new(expired_projectors_count).encode_ape();

            // 8.3 Extend the payload bits with the expired projectors count bits.
            payload_bits.extend(expired_projectors_count_bits);
        }

        // 9 Retrieve the new projector.
        // Not set for the time being.
        let new_projector: Option<Projector> = None;

        // 10 Insert a bit to the beginning of the payload bits to indicate the presence of the new projector.
        match new_projector {
            // 10.a The new projector is set.
            Some(_) => {
                // 10.a.1 Push true bit to indicate the presence of the projector.
                payload_bits.push(true);
            }
            // 10.b The new projector is not set.
            None => {
                // 10.b.1 Push false bit to indicate the absence of the projector.
                payload_bits.push(false);
            }
        }

        // 11 Encode the added entries.
        for entry in &self.added_entries {
            // 11.1 Encode the entry as APE bits.
            let entry_ape_bits = entry
                .encode_ape(
                    batch_height,
                    &self.registery,
                    encode_account_rank_as_longval,
                    encode_contract_rank_as_longval,
                )
                .await
                .map_err(IntoBatchContainerError::EntryAPEEncodeError)?;

            // 11.2 Extend the payload bits with the entry APE bits.
            payload_bits.extend(entry_ape_bits);
        }

        // 12 Convert the payload bits to payload bytes.
        let new_payload_bytes: Bytes = payload_bits.to_ape_payload_bytes();

        // 13 Get prev projectors from sync manager: Not implemented for the time being.
        let prev_projectors = Vec::<Projector>::new();

        // 14 Get the executed entries.
        let executed_entries: Vec<Entry> = self.added_entries.clone();

        // 15 Construct the new payload.
        let new_payload = Payload::new(self.engine_key, new_payload_bytes.clone(), None);

        // 16 Construct the signed batch transaction.
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

        // 17 Construct the batch container.
        let batch_container =
            BatchContainer::new(batch_height, new_payload_bytes, signed_batch_txn);

        // 18 Return the batch container.
        Ok(batch_container)
    }

    /// Executes a `Liftup` entry in the `SessionPool`.
    pub async fn exec_liftup_in_pool(
        &mut self,
        liftup: &Liftup,
        liftup_bls_signature: [u8; 96],
    ) -> Result<(EntryId, Entry,  BatchHeight, BatchTimestamp), ExecLiftupInPoolError> {
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
            // 1.c The session has taken a break.
            SessionPoolState::Break => {
                return Err(ExecLiftupInPoolError::SessionBreakError);
            }
            // 1.d The session is active but it might be overloaded.
            _ => {
                // 1.d.1 Check if the pool is overloaded.
                if self.added_entries.len() >= MAX_IN_POOL_ENTRIES {
                    return Err(ExecLiftupInPoolError::PoolOverloadedError);
                }
            }
        };

        // 2 Get the batch height and batch timestamp.
        let (batch_height, batch_timestamp, _) = self
            .batch_info
            .ok_or(ExecLiftupInPoolError::BatchInfoNotFoundError)?;

        // 3 Run pre-validations.
        {
            liftup
                .validate_overall(
                    self.engine_key,
                    batch_height,
                    &self.utxo_set,
                    &self.registery,
                    &self.graveyard,
                    liftup_bls_signature,
                )
                .await
                .map_err(|err| {
                    ExecLiftupInPoolError::LiftupValidateOverallError(format!("{err:?}"))
                })?;
        }

        // 4 Prepare for the execution by backing up the execution context.
        {
            let mut _exec_ctx = self.exec_ctx.lock().await;
            _exec_ctx.pre_execution().await;
        }

        // 5 Execute the liftup in the execution context.
        // Drop the mutex guard before `match` arms run — otherwise `rollback_last` would re-lock
        // the same mutex and deadlock (scrutinee temporaries live until the whole `match` ends).
        let liftup_result = {
            let mut exec_ctx = self.exec_ctx.lock().await;
            exec_ctx.execute_liftup(liftup, batch_timestamp).await
        };

        match liftup_result {
            // 5.a Success.
            Ok(liftup_entry) => {
                // 5.a.1 Derive the entry id.
                let entry_index_in_batch = self.added_entries.len() as u32;
                let entry_id = liftup_entry
                    .entry_id(batch_height, entry_index_in_batch)
                    .ok_or(ExecLiftupInPoolError::EntryIdDerivationError)?;

                // 5.a.2 Add the liftup entry to the added entries.
                self.added_entries.push(liftup_entry.clone());

                // 5.a.3 Add the liftup BLS signature to the added individual entry BLS signatures.
                self.added_individual_entry_bls_signatures
                    .push(liftup_bls_signature);

                // 5.a.4 Return the liftup entry and pool metadata.
                Ok((entry_id, liftup_entry, batch_height, batch_timestamp))
            }

            // 5.b Error.
            Err(error) => {
                // 5.b.1 Rollback the execution.
                {
                    self.exec_ctx.lock().await.rollback_last().await;
                }

                // 5.b.2 Return the error.
                Err(ExecLiftupInPoolError::LiftupExecutionError(format!(
                    "{error:?}"
                )))
            }
        }
    }

    /// Executes a `Move` entry in the `SessionPool`.
    pub async fn exec_move_in_pool(
        &mut self,
        move_entry: &Move,
        move_bls_signature: [u8; 96],
    ) -> Result<(EntryId, Entry, BatchHeight, BatchTimestamp), ExecMoveInPoolError> {
        // 1 Check the pool session status.
        match self.state {
            SessionPoolState::Inactive => {
                return Err(ExecMoveInPoolError::SessionInactiveError);
            }
            SessionPoolState::Suspended => {
                return Err(ExecMoveInPoolError::SessionSuspendedError);
            }
            SessionPoolState::Break => {
                return Err(ExecMoveInPoolError::SessionBreakError);
            }
            _ => {
                if self.added_entries.len() >= MAX_IN_POOL_ENTRIES {
                    return Err(ExecMoveInPoolError::PoolOverloadedError);
                }
            }
        };

        // 2 Get the batch height and batch timestamp.
        let (batch_height, batch_timestamp, _) = self
            .batch_info
            .ok_or(ExecMoveInPoolError::BatchInfoNotFoundError)?;
        // 3 Verify entry signature and run pre-validations.
        {
            move_entry
                .bls_verify(move_bls_signature)
                .map_err(|err| ExecMoveInPoolError::MoveBLSVerifyError(format!("{err:?}")))?;

            move_entry
                .validate_overall(
                    batch_height,
                    &self.registery,
                    &self.graveyard,
                    &self.coin_manager,
                )
                .await
                .map_err(|err| ExecMoveInPoolError::MoveValidateOverallError(format!("{err:?}")))?;
        }
        // 4 Prepare for execution by backing up the execution context.
        {
            let mut _exec_ctx = self.exec_ctx.lock().await;
            _exec_ctx.pre_execution().await;
        }
        // 5 Execute the move in the execution context.
        // Drop the mutex guard before `match` arms — see `exec_liftup_in_pool` for deadlock note.
        let move_result = {
            let mut exec_ctx = self.exec_ctx.lock().await;
            exec_ctx.execute_move(move_entry, batch_timestamp).await
        };

        match move_result {
            // 5.a Success.
            Ok(move_entry_wrapped) => {
                // 5.a.1 Derive the entry id.
                let entry_index_in_batch = self.added_entries.len() as u32;
                let entry_id = move_entry_wrapped
                    .entry_id(batch_height, entry_index_in_batch)
                    .ok_or(ExecMoveInPoolError::EntryIdDerivationError)?;

                // 5.a.2 Add the move entry to the added entries.
                self.added_entries.push(move_entry_wrapped.clone());

                // 5.a.3 Add move BLS signature to pooled individual entry signatures.
                self.added_individual_entry_bls_signatures
                    .push(move_bls_signature);

                // 5.a.4 Return entry and pool metadata.
                Ok((entry_id, move_entry_wrapped, batch_height, batch_timestamp))
            }

            // 5.b Error.
            Err(error) => {
                // 5.b.1 Rollback the execution.
                {
                    self.exec_ctx.lock().await.rollback_last().await;
                }
                // 5.b.2 Return the error.
                Err(ExecMoveInPoolError::MoveExecutionError(format!("{error:?}")))
            }
        }
    }
}
