#[derive(Debug, Clone)]
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
