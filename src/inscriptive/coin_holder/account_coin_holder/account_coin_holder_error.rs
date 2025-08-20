/// The state construction error.
#[derive(Debug, Clone)]
pub enum AccountCoinHolderConstructionError {
    // DB open error.
    DBOpenError(sled::Error),
    // Key deserialize at index error.
    RegisteryIndexDeserializeErrorAtIndex(usize),
    // Value deserialize at index error.
    CoinBalanceDeserializeErrorAtIndex(usize),
}
