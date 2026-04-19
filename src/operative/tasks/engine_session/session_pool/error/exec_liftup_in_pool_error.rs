/// Errors associated with executing a `Liftup` entry in the `SessionPool`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecLiftupInPoolError {
    /// The session is inactive.
    SessionInactiveError,
    SessionSuspendedError,
    SessionBreakError,
    PoolOverloadedError,
    BatchInfoNotFoundError,
    LiftupValidateOverallError(String),
    LiftupExecutionError(String),
    /// The entry ID could not be derived for the executed entry.
    EntryIdDerivationError,
}
