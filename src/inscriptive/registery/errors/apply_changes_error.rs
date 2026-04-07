/// Account Key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with applying changes to the `RegisteryManager`.
#[derive(Debug, Clone)]
pub enum RMApplyChangesError {
    AccountTreeOpenError(AccountKey, sled::Error),
    AccountRegisteryIndexInsertError(AccountKey, u64, sled::Error),
    AccountCallCounterInsertError(AccountKey, u64, sled::Error),
    AccountBLSKeyInsertError(AccountKey, sled::Error),
    AccountSecondaryAggregationKeyInsertError(AccountKey, sled::Error),
    AccountNotFoundInMemory(AccountKey),
    AccountCallCounterUpdateError(AccountKey, u64, sled::Error),
    ContractTreeOpenError(ContractId, sled::Error),
    ContractRegisteryIndexInsertError(ContractId, u64, sled::Error),
    ContractCallCounterInsertError(ContractId, u64, sled::Error),
    ContractProgramBytesInsertError(ContractId, sled::Error),
    ContractNotFoundInMemory(ContractId),
    ContractCallCounterUpdateError(ContractId, u64, sled::Error),
    ExecutableCompileError(ContractId, crate::executive::executable::compiler::compiler_error::ExecutableCompileError),
}
