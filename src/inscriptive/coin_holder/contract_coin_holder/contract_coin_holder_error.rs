/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// State key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = Vec<u8>;

/// State value.
#[allow(non_camel_case_types)]
type ALLOC_BALANCE = u64;

/// The state construction error.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderConstructionError {
    // Main DB open error.
    BalancesDBOpenError(sled::Error),
    // Key deserialize at index error.
    ContractIDDeserializeErrorAtIndex(usize),
    // Value deserialize at index error.    
    CoinBalanceDeserializeErrorAtIndex(usize),
    // Sub DB open error.
    ShadowAllocDBOpenError(sled::Error),
    ShadowSpaceDBOpenError(sled::Error),
    ShadowSpaceTreeOpenError(CONTRACT_ID, sled::Error),
    InvalidContractIDBytes(Vec<u8>),
    ContractShadowIterError(sled::Error),
    InvalidShadowAccountKey(Vec<u8>),
    InvalidShadowBalance(Vec<u8>),
    InvalidContractBalance(Vec<u8>),
    InvalidShadowAllocation(Vec<u8>),
    BalanceGetError(CONTRACT_ID, sled::Error),
    ShadowAllocationGetError(CONTRACT_ID, sled::Error),
}

/// The state save error.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderSaveError {
    OpenTreeError(CONTRACT_ID, sled::Error),
    TreeValueInsertError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        ALLOC_BALANCE,
        sled::Error,
    ),
}
