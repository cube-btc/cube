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
    UnableToDeserializeAccountLivenessFlagFromBytes(AccountKey, Vec<u8>),
    InvalidAccountHierarchyBytecode(AccountKey, Vec<u8>),
    UnableToDeserializeAccountHierarchyFromBytecode(AccountKey, u8),
    UnableToDeserializeAccountTxFeeExemptionsFromBytes(AccountKey, Vec<u8>),
    UnableToDeserializeAccountCanDeployLiquidityFromBytes(AccountKey, Vec<u8>),
    UnableToDeserializeAccountCanDeployContractFromBytes(AccountKey, Vec<u8>),    
    UnableToDeserializeAccountLastActivityTimestampFromBytes(AccountKey, Vec<u8>),
    AccountLivenessFlagNotPresent(AccountKey),
    AccountHierarchyNotPresent(AccountKey),
    AccountTxFeeExemptionsNotPresent(AccountKey),
    AccountCanDeployLiquidityNotPresent(AccountKey),
    AccountCanDeployContractNotPresent(AccountKey),
    AccountLastActivityTimestampNotPresent(AccountKey),
    // Contract related errors.
    UnableToDeserializeContractKeyBytesFromTreeName(Vec<u8>),
    ContractsTreeOpenError(ContractId, sled::Error),
    ContractsTreeIterError(ContractId, sled::Error),
    UnableToDeserializeContractKeyByteFromTreeKey(ContractId, Vec<u8>),
    InvalidContractDbKeyByte(ContractId, Vec<u8>),
    UnableToDeserializeContractLivenessFlagFromBytes(ContractId, Vec<u8>),
    InvalidContractImmutabilityBytecode(ContractId, Vec<u8>),
    UnableToDeserializeContractTaxExemptionsFromBytes(ContractId, Vec<u8>),
    UnableToDeserializeContractStorageLimitFromBytes(ContractId, Vec<u8>),
    UnableToDeserializeContractLastActivityTimestampFromBytes(ContractId, Vec<u8>),
    ContractLivenessFlagNotPresent(ContractId),
    ContractImmutabilityNotPresent(ContractId),
    ContractTaxExemptionsNotPresent(ContractId),
    ContractStorageLimitNotPresent(ContractId),
    ContractLastActivityTimestampNotPresent(ContractId),
}
