/// Account key.
type AccountKey = [u8; 32];

/// Rollup height.
type AtRollupHeight = u64;

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

    /// Global flame set database related errors.
    /// ------------------------------------------------------------
    GlobalFlameSetDBOpenError(sled::Error),
    UnableToDeserializeRollupHeightBytesFromTreeName(Vec<u8>),
    GlobalFlameSetTreeOpenError(AtRollupHeight, sled::Error),
    GlobalFlameSetTreeIterError(AtRollupHeight, sled::Error),
    UnableToDeserializeFlameIndexBytesFromTreeKey(AtRollupHeight, Vec<u8>),
    UnableToDeserializeFlameBytesFromTreeValue(AtRollupHeight, Vec<u8>),
}
