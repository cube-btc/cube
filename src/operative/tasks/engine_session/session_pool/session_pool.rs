use crate::constructive::bitcoiny::batch_template::batch_template::BatchTemplate;
use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::txout_types::projector::projector::Projector;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::executive::exec_ctx::exec_ctx::EXEC_CTX;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::tasks::engine_session::session_pool::error::exec_liftup_in_pool_error::ExecLiftupInPoolError;
use crate::operative::tasks::engine_session::session_pool::error::into_batch_template_error::IntoBatchTemplateError;
use crate::transmutative::bls::agg::bls_aggregate;
use crate::transmutative::codec::bitvec_ext::BitVecExt;
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
        utxo_set: &UTXO_SET,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
        coin_manager: &COIN_MANAGER,
        flame_manager: &FLAME_MANAGER,
    ) -> SESSION_POOL {
        // 1 Construct the exec container.
        let exec_container = ExecCtx::construct(
            engine_key,
            Arc::clone(utxo_set),
            Arc::clone(registery),
            Arc::clone(graveyard),
            Arc::clone(coin_manager),
            Arc::clone(flame_manager),
        );

        // 2 Construct the session pool.
        let session_pool = SessionPool {
            state: SessionPoolState::Inactive,
            engine_key,
            utxo_set: Arc::clone(utxo_set),
            registery: Arc::clone(registery),
            graveyard: Arc::clone(graveyard),
            coin_manager: Arc::clone(coin_manager),
            flame_manager: Arc::clone(flame_manager),
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
    pub async fn flush(&mut self) {
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
    pub fn end_session(&mut self) {
        self.state = SessionPoolState::Inactive;
    }

    /// Aggregates the BLS signatures of the added entries.
    pub fn aggregate_bls_signature(&self) -> Result<[u8; 96], BLSError> {
        bls_aggregate(self.added_individual_entry_bls_signatures.clone())
    }

    /// Converts the `ExecCtx` into a `BatchTemplate`.
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

        // 4 Encode the aggregate BLS signature.
        {
            // 4.1 Get the aggregate BLS signature.
            let aggregate_bls_signature = self
                .aggregate_bls_signature()
                .map_err(|_| IntoBatchTemplateError::AggregateBLSSignatureError)?;

            // 4.2 Convert the aggregate BLS signature to bits.
            let aggregate_bls_signature_bits = BitVec::from_bytes(&aggregate_bls_signature);

            // 4.3 Extend the payload bits with the aggregate BLS signature bits.
            payload_bits.extend(aggregate_bls_signature_bits);
        }

        // 5 Retrieve the projector.
        // Not set for the time being.
        let projector: Option<Projector> = None;

        // 6 Insert a bit to the beginning of the payload bits to indicate the presence of the projector.
        match projector {
            // 6.a The projector is set.
            Some(_) => {
                // 6.a.1 Push true bit to indicate the presence of the projector.
                payload_bits.push(true);
            }
            // 6.b The projector is not set.
            None => {
                // 6.b.1 Push false bit to indicate the absence of the projector.
                payload_bits.push(false);
            }
        }

        // 7 Encode the added entries.
        for entry in &self.added_entries {
            // 7.1 Encode the entry as APE bits.
            let entry_ape_bits = entry
                .encode_ape(batch_height, &self.registery, true, true)
                .await
                .map_err(IntoBatchTemplateError::EntryAPEEncodeError)?;

            // 7.2 Extend the payload bits with the entry APE bits.
            payload_bits.extend(entry_ape_bits);
        }

        // 8 Convert the payload bits to payload bytes.
        let payload_bytes: Bytes = payload_bits.to_ape_payload_bytes();

        // 9 Collect the Bitcoin transaction inputs.
        let bitcoin_tx_inputs: Vec<OutPoint> = self
            .added_tx_inputs
            .iter()
            .map(|(outpoint, _)| outpoint.clone())
            .collect();

        // 10 Get the Bitcoin transaction outputs.
        let bitcoin_tx_outputs: Vec<TxOut> = self.added_tx_outputs.clone();

        // 11 Construct the batch template.
        let batch_template =
            BatchTemplate::new(bitcoin_tx_inputs, bitcoin_tx_outputs, payload_bytes);

        // 12 Return the batch template.
        Ok(batch_template)
    }

    /// Executes a `Liftup` entry in the `SessionPool`.
    pub async fn exec_liftup_in_pool(
        &mut self,
        execution_batch_height: u64,
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

        // 2 Run pre-validations.
        {
            liftup
                .validate_overall(
                    self.engine_key,
                    execution_batch_height,
                    &self.utxo_set,
                    &self.registery,
                    &self.graveyard,
                    liftup_bls_signature,
                )
                .await
                .map_err(|err| ExecLiftupInPoolError::LiftupValidateOverallError(err))?;
        }

        // 3 Prepare for the execution by backing up the execution context.
        {
            let mut _exec_container = self.exec_container.lock().await;
            _exec_container.pre_execution().await;
        }

        // 4 Execute the liftup in the execution context.
        match self
            .exec_container
            .lock()
            .await
            .execute_liftup(liftup, execution_timestamp)
            .await
        {
            // 4.a Success.
            Ok(liftup_entry) => {
                // 4.a.1 Add the liftup entry to the added entries.
                self.added_entries.push(liftup_entry.clone());

                // 4.a.2 Add the liftup BLS signature to the added individual entry BLS signatures.
                self.added_individual_entry_bls_signatures
                    .push(liftup_bls_signature);

                // 4.a.3 Add internal Lifts inside the Liftup to the added Bitcoin transaction inputs.
                {
                    self.added_tx_inputs.extend(
                        liftup
                            .lift_tx_inputs
                            .iter()
                            .map(|lift| (lift.outpoint(), lift.txout().clone())),
                    );
                }

                // 4.a.4 Return the liftup entry.
                Ok(liftup_entry)
            }

            // 4.b Error.
            Err(error) => {
                // 4.b.1 Rollback the execution.
                {
                    self.exec_container.lock().await.rollback_last().await;
                }

                // 4.b.2 Return the error.
                Err(ExecLiftupInPoolError::LiftupExecutionError(error))
            }
        }
    }
}
