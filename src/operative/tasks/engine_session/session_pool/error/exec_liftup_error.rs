use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;

/// Errors associated with executing a `Liftup` entry in the `SessionPool`.
#[derive(Debug, Clone)]
pub enum ExecLiftupInPoolError {
    /// The session is inactive.
    SessionInactiveError,
    SessionSuspendedError,
    PoolOverloadedError,
    LiftupExecutionError(LiftupExecutionError),
}
