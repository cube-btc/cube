/// Account key.
type AccountKey = [u8; 32];

/// Flame db key.
type FlameDbKey = [u8; 12];

/// Rollup height.
type ProjectorHeight = u64;

/// Flame index.
type FlameIndex = u32;

/// Errors associated with applying changes to the `FlameManager`.
#[derive(Debug, Clone)]
pub enum FMApplyChangesError {
    ProjectorExpiryHeightIsNotSet,
    AccountTreeOpenError(AccountKey, sled::Error),
    AccountTargetFlameValueCouldNotBeRetrieved(AccountKey),
    AccountRemoveFlameFromDiskTreeError(AccountKey, FlameDbKey, sled::Error),
    AccountInsertFlameIntoDiskTreeError(AccountKey, FlameDbKey, sled::Error),
    GlobalFlameSetTreeDropError(ProjectorHeight, sled::Error),
    GlobalFlameSetTreeOpenError(ProjectorHeight, sled::Error),
    GlobalFlameSetInsertFlameIntoDiskTreeError(ProjectorHeight, FlameIndex, sled::Error),
}
