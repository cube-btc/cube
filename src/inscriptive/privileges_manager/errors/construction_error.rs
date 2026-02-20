/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with constructing the `PrivilegesManager`.
#[derive(Debug, Clone)]
pub enum PrivilegesManagerConstructionError {
    AccountsDBOpenError(sled::Error),
    ContractsDBOpenError(sled::Error),
    // Account related errors.
    UnableToDeserializeAccountKeyBytesFromTreeName(Vec<u8>),
    AccountsTreeOpenError(AccountKey, sled::Error),
    AccountsTreeIterError(AccountKey, sled::Error),
    UnableToDeserializeAccountKeyByteFromTreeKey(AccountKey, Vec<u8>),
    InvalidAccountDbKeyByte(AccountKey, Vec<u8>),
    // Account elements
    InvalidAccountHierarchyBytecode(AccountKey, Vec<u8>),
    UnableToDeserializeAccountHierarchyFromBytecode(AccountKey, u8),
    UnableToDeserializeAccountLivenessFlagFromBytes(AccountKey, Vec<u8>),
    UnableToDeserializeAccountLastActivityTimestampFromBytes(AccountKey, Vec<u8>),
    UnableToDeserializeAccountTxFeePrivilegesFromBytes(AccountKey, Vec<u8>),
    UnableToDeserializeAccountTransactingLimitsFromBytes(AccountKey, Vec<u8>),
    InvalidAccountCanDeployLiquidityBytes(AccountKey, Vec<u8>),
    InvalidAccountCanDeployContractBytes(AccountKey, Vec<u8>),
    // Not present errors.
    AccountHierarchyNotPresent(AccountKey),
    AccountLivenessFlagNotPresent(AccountKey),
    AccountLastActivityTimestampNotPresent(AccountKey),
    AccountTxFeePrivilegesNotPresent(AccountKey),
    AccountTransactingLimitsNotPresent(AccountKey),
    AccountCanDeployLiquidityNotPresent(AccountKey),
    AccountCanDeployContractNotPresent(AccountKey),
    // Contract related errors.
    UnableToDeserializeContractKeyBytesFromTreeName(Vec<u8>),
}
