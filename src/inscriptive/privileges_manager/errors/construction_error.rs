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
    UnableToDeserializeAccountLivenessFlagFromBytes(AccountKey, Vec<u8>),

    InvalidAccountHierarchyBytecode(AccountKey, Vec<u8>),
    UnableToDeserializeAccountHierarchyFromBytecode(AccountKey, u8),
    UnableToDeserializeAccountTxFeeExemptionsFromBytes(AccountKey, Vec<u8>),
    UnableToDeserializeAccountCanDeployLiquidityFromBytes(AccountKey, Vec<u8>),
    UnableToDeserializeAccountCanDeployContractFromBytes(AccountKey, Vec<u8>),    
    UnableToDeserializeAccountLastActivityTimestampFromBytes(AccountKey, Vec<u8>),
    // Not present errors.
    AccountLivenessFlagNotPresent(AccountKey),
    AccountHierarchyNotPresent(AccountKey),
    AccountTxFeeExemptionsNotPresent(AccountKey),
    AccountCanDeployLiquidityNotPresent(AccountKey),
    AccountCanDeployContractNotPresent(AccountKey),
    AccountLastActivityTimestampNotPresent(AccountKey),
    // Contract related errors.
    UnableToDeserializeContractKeyBytesFromTreeName(Vec<u8>),
}
