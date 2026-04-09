use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::executive::exec_container::exec_container::ExecContainer;
use crate::executive::exec_container::exec_container::EXEC_CONTAINER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The state of the `SessionPool`.
pub enum SessionPoolState {
    // The session pool is inactive.
    Inactive,
    // The session pool is active.
    Active,
    // The session pool is overloaded so no new entries can be added.
    Overloaded,
    // The pool is suspended for some reason.
    Suspended,
}

/// `SessionPool` represents the local mempool of the Engine, containing a collection of entries that have been locally executed and pooled.
pub struct SessionPool {
    // The state of the session pool.
    pub state: SessionPoolState,

    // The exec container.
    pub exec_container: EXEC_CONTAINER,
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
        let exec_container = ExecContainer::construct(
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
        };

        // 3 Return the guarded `SessionPool`.
        Arc::new(Mutex::new(session_pool))
    }

    /// Starts the session of the `SessionPool`.
    pub fn begin_session(&mut self) {
        self.state = SessionPoolState::Active;
    }

    /// Suspends the session of the `SessionPool`.
    pub fn suspend_session(&mut self, overloaded: bool) {
        // Suspend the session according to the appropriate state.
        if overloaded {
            self.state = SessionPoolState::Overloaded;
        } else {
            self.state = SessionPoolState::Suspended;
        }
    }

    /// Resumes the session of the `SessionPool`.
    pub fn resume_session(&mut self) {
        self.state = SessionPoolState::Active;
    }

    /// Ends the session of the `SessionPool`.
    pub fn end_session(&mut self) {
        // 1 Deactivate the `SessionPool`.
        self.state = SessionPoolState::Inactive;
    }

    /// Executes a `Liftup` entry in the pool.
    pub async fn exec_liftup(&mut self, _liftup: Liftup) {}
}
