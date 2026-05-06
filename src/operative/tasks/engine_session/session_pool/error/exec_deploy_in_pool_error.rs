/// Errors associated with executing a `Deploy` entry in the `SessionPool`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecDeployInPoolError {
    SessionInactiveError,
    SessionSuspendedError,
    SessionBreakError,
    PoolOverloadedError,
    BatchInfoNotFoundError,
    DeployBLSVerifyError(String),
    DeployValidateMethodsError(String),
    DeployValidateRootAccountError(String),
    DeployValidateTargetError {
        targeted_at_batch_height: u64,
        execution_batch_height: u64,
    },
    DeployExecutionError(String),
    EntryIdDerivationError,
}
