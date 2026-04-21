/// Errors associated with executing a `Move` entry in the `SessionPool`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecMoveInPoolError {
    /// The session is inactive.
    SessionInactiveError,
    SessionSuspendedError,
    SessionBreakError,
    PoolOverloadedError,
    BatchInfoNotFoundError,
    MoveBLSVerifyError(String),
    MoveValidateOverallError(String),
    MoveExecutionError(String),
    /// The entry ID could not be derived for the executed entry.
    EntryIdDerivationError,
}
