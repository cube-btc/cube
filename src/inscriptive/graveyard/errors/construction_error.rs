/// Errors associated with constructing the `Graveyard`.
#[derive(Debug, Clone)]
pub enum GraveyardConstructionError {
    DBOpenError(sled::Error),
    UnableToDeserializeAccountKeyBytesFromDBKey(Vec<u8>),
    UnableToDeserializeRedemptionAmountBytesFromDBValue(Vec<u8>, Vec<u8>),
}
