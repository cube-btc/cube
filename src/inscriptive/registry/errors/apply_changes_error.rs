/// Account Key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with applying changes to the `RegistryManager`.
#[derive(Debug, Clone)]
pub enum RMApplyChangesError {
    AccountTreeOpenError(AccountKey, sled::Error),
    AccountRegistryIndexInsertError(AccountKey, u64, sled::Error),
    AccountCallCounterInsertError(AccountKey, u64, sled::Error),
    AccountLastActivityTimestampInsertError(AccountKey, u64, sled::Error),
    AccountBLSKeyInsertError(AccountKey, sled::Error),
    AccountSecondaryAggregationKeyInsertError(AccountKey, sled::Error),
    AccountFlameConfigInsertError(AccountKey, sled::Error),
    AccountProjectorConfigInsertError(AccountKey, sled::Error),
    AccountNotFoundInMemory(AccountKey),
    AccountCallCounterUpdateError(AccountKey, u64, sled::Error),
    AccountLastActivityTimestampUpdateError(AccountKey, u64, sled::Error),
    ContractTreeOpenError(ContractId, sled::Error),
    ContractRegistryIndexInsertError(ContractId, u64, sled::Error),
    ContractCallCounterInsertError(ContractId, u64, sled::Error),
    ContractLastActivityTimestampInsertError(ContractId, u64, sled::Error),
    ContractProgramBytesInsertError(ContractId, sled::Error),
    ContractNotFoundInMemory(ContractId),
    ContractCallCounterUpdateError(ContractId, u64, sled::Error),
    ContractLastActivityTimestampUpdateError(ContractId, u64, sled::Error),
    ProgramCompileError(ContractId, crate::executive::executable::compiler::compiler_error::ProgramCompileError),
}
