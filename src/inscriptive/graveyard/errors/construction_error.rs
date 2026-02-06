/// Errors associated with constructing the `Graveyard`.
#[derive(Debug, Clone)]
pub enum GraveyardConstructionError {
    DBOpenError(sled::Error),
    UnableToDeserializeAccountKeyBytesFromTreeName(Vec<u8>),
    UnableToDeserializeSatoshiRedemptionAmountBytesFromTreeValue(Vec<u8>, Vec<u8>),
}
