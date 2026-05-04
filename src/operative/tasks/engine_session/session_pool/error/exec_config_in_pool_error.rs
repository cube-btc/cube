/// Errors associated with executing a `Config` entry in the `SessionPool`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecConfigInPoolError {
    SessionInactiveError,
    SessionSuspendedError,
    SessionBreakError,
    PoolOverloadedError,
    BatchInfoNotFoundError,
    ConfigBLSVerifyError(String),
    ConfigValidateRootAccountError(String),
    ConfigValidateTargetError {
        targeted_at_batch_height: u64,
        execution_batch_height: u64,
    },
    ConfigExecutionError(String),
    EntryIdDerivationError,
}
