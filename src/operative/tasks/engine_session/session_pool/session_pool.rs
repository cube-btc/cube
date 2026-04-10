use crate::constructive::bitcoiny::batch_template::batch_template::BatchTemplate;
use crate::constructive::entry::entry::Entry;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::executive::exec_ctx::errors::into_batch_template_error::IntoBatchTemplateError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::executive::exec_ctx::exec_ctx::EXEC_CTX;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use crate::operative::tasks::engine_session::session_pool::error::exec_liftup_error::ExecLiftupInPoolError;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    // The exec container.
    pub exec_container: EXEC_CTX,

    // The number of entries in the pool.
    pub in_pool_entries_count: usize,
}

/// Guarded `SessionPool`.
#[allow(non_camel_case_types)]
pub type SESSION_POOL = Arc<Mutex<SessionPool>>;

impl SessionPool {
    /// Constructs the `SessionPool`.    
    pub fn construct(
        engine_key: [u8; 32],
        utxo_set: UTXO_SET,
        registery: REGISTERY,
        graveyard: GRAVEYARD,
        coin_manager: COIN_MANAGER,
        flame_manager: FLAME_MANAGER,
    ) -> SESSION_POOL {
        // 1 Construct the exec container.
        let exec_container = ExecCtx::construct(
            engine_key,
            utxo_set,
            registery,
            graveyard,
            coin_manager,
            flame_manager,
        );

        // 2 Construct the session pool.
        let session_pool = SessionPool {
            state: SessionPoolState::Inactive,
            exec_container,
            in_pool_entries_count: 0,
        };

        // 3 Return the guarded `SessionPool`.
        Arc::new(Mutex::new(session_pool))
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

        // 2 Reset the number of entries in the pool.
        self.in_pool_entries_count = 0;
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

    /// Returns the `BatchTemplate` of locally executed Entries in the pool.
    pub async fn batch_template(&mut self, batch_height: u64, batch_timestamp: u64, payload_version: u32) -> Result<BatchTemplate, IntoBatchTemplateError> {
        // 1 Convert the `ExecCtx` into a `BatchTemplate`.
        self.exec_container.lock().await.into_batch_template(batch_height, batch_timestamp, payload_version).await
    }

    /// Executes a `Liftup` entry in the `SessionPool`.
    pub async fn exec_liftup_in_pool(
        &mut self,
        liftup: Liftup,
        session_timestamp: u64,
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
                if self.in_pool_entries_count >= MAX_IN_POOL_ENTRIES {
                    return Err(ExecLiftupInPoolError::PoolOverloadedError);
                }
            }
        };

        // 2 Prepare for the execution by backing up the execution context.
        {
            let mut _exec_container = self.exec_container.lock().await;
            _exec_container.pre_execution().await;
        }

        // 3 Execute the liftup in the execution context.
        match self
            .exec_container
            .lock()
            .await
            .execute_liftup(liftup, session_timestamp)
            .await
        {
            // 3.a Success.
            Ok(liftup_entry) => {
                // 3.a.1 Increment the number of entries in the pool.
                self.in_pool_entries_count += 1;

                // 3.a.2 Return the liftup entry.
                Ok(liftup_entry)
            }

            // 3.b Error.
            Err(error) => {
                // 3.b.1 Rollback the execution.
                {
                    self.exec_container.lock().await.rollback_last().await;
                }

                // 3.b.2 Return the error.
                Err(ExecLiftupInPoolError::LiftupExecutionError(error))
            }
        }
    }
}
