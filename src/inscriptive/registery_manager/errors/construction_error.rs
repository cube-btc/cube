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
    UnableToDeserializeAccountKeyBytesFromTreeName(Vec<u8>),
    AccountsTreeOpenError(AccountKey, sled::Error),
    AccountsTreeIterError(AccountKey, sled::Error),

    /// Contract related errors.
    /// ------------------------------------------------------------
    ContractsDBOpenError(sled::Error),
    ContractsTreeOpenError(ContractId, sled::Error),
    ContractsTreeIterError(ContractId, sled::Error),
}
