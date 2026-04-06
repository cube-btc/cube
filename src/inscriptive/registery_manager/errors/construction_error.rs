use crate::executive::executable::compiler::compiler_error::ExecutableDecompileError;

/// Account Key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with constructing `RegisteryManager`.
#[derive(Debug, Clone)]
pub enum RMConstructionError {
    /// Account related errors.
    /// ------------------------------------------------------------
    AccountsDBOpenError(sled::Error),
    AccountsTreeOpenError(AccountKey, sled::Error),
    AccountsTreeIterError(AccountKey, sled::Error),
    UnableToDeserializeAccountDbKeyByteFromTreeKey(AccountKey, Vec<u8>),
    UnableToDeserializeAccountRegisteryIndexBytesFromTreeValue(AccountKey, Vec<u8>),
    UnableToDeserializeAccountCallCounterBytesFromTreeValue(AccountKey, Vec<u8>),
    UnableToDeserializeAccountPrimaryBLSKeyBytesFromTreeValue(AccountKey, Vec<u8>),
    UnableToDeserializeAccountSecondaryAggregationKeyBytesFromTreeValue(AccountKey, Vec<u8>),
    InvalidAccountDbKeyByte(AccountKey, Vec<u8>),

    /// Contract related errors.
    /// ------------------------------------------------------------
    ContractsDBOpenError(sled::Error),
    ContractsTreeOpenError(ContractId, sled::Error),
    ContractsTreeIterError(ContractId, sled::Error),
    UnableToDeserializeContractDbKeyByteFromTreeKey(ContractId, Vec<u8>),
    UnableToDeserializeContractRegisteryIndexBytesFromTreeValue(ContractId, Vec<u8>),
    UnableToDeserializeContractCallCounterBytesFromTreeValue(ContractId, Vec<u8>),
    ContractExecutableDecompileError(ContractId, ExecutableDecompileError),
    InvalidContractDbKeyByte(ContractId, Vec<u8>),
}
