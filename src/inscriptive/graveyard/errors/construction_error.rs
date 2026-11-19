/// Errors associated with constructing the `Graveyard`.
#[derive(Debug, Clone)]
pub enum GraveyardConstructionError {
    AccountsDBOpenError(sled::Error),
    ContractsDBOpenError(sled::Error),
    UnableToDeserializeAccountKeyBytesFromDBKey(Vec<u8>),
    UnableToDeserializeRedemptionAmountBytesFromDBValue(Vec<u8>, Vec<u8>),
    UnableToDeserializeContractIdBytesFromDBKey(Vec<u8>),
}
