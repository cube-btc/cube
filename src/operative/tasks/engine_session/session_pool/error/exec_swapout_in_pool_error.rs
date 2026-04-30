#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecSwapoutInPoolError {
    SessionInactiveError,
    SessionSuspendedError,
    SessionBreakError,
    PoolOverloadedError,
    BatchInfoNotFoundError,
    SwapoutValidateOverallError(String),
    EntryIdDerivationError,
    SwapoutExecutionError(String),
}
