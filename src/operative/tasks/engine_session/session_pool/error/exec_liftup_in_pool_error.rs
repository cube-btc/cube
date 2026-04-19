use crate::constructive::entry::entry_kinds::liftup::ext::pre_validations::validate_overall::validate_overall_error::LiftupValidateOverallError;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;

/// Errors associated with executing a `Liftup` entry in the `SessionPool`.
#[derive(Debug, Clone)]
pub enum ExecLiftupInPoolError {
    /// The session is inactive.
    SessionInactiveError,
    SessionSuspendedError,
    SessionBreakError,
    PoolOverloadedError,
    BatchInfoNotFoundError,
    LiftupValidateOverallError(LiftupValidateOverallError),
    LiftupExecutionError(LiftupExecutionError),
}
