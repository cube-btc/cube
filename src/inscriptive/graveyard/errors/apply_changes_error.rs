/// Account key.
type AccountKey = [u8; 32];

/// Errors associated with applying changes to the `Graveyard`.
#[derive(Debug, Clone)]
pub enum GraveyardApplyChangesError {
    /// Error when resetting redemption amount on disk.
    RedemptionAmountResetDBError(AccountKey, sled::Error),

    /// Error when inserting buried account on disk.
    BurriedAccountInsertDBError(AccountKey, sled::Error),
}
