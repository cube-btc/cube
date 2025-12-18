/// Account key.
type AccountKey = [u8; 32];

/// Errors associated with constructing the `FlameManager`.
#[derive(Debug, Clone)]
pub enum FMConstructionError {
    /// Accounts database related errors.
    /// ------------------------------------------------------------
    AccountsDBOpenError(sled::Error),
    UnableToDeserializeAccountKeyBytesFromTreeName(Vec<u8>),
    AccountsTreeOpenError(AccountKey, sled::Error),
    AccountsTreeIterError(AccountKey, sled::Error),
    UnableToDeserializeAccountDbKeyByteFromTreeKey(AccountKey, Vec<u8>),
    UnableToDeserializeAccountFlameConfigBytesFromTreeValue(AccountKey, Vec<u8>),
    UnableToDeserializeAccountFlameSetBytesFromTreeValue(AccountKey, Vec<u8>),
    InvalidAccountDbKeyByte(AccountKey, Vec<u8>),
}
