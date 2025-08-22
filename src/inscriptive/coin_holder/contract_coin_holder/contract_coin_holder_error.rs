/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// State key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// State value.
#[allow(non_camel_case_types)]
type ALLOCS_SUM = u64;

/// State value.
#[allow(non_camel_case_types)]
type CONTRACT_BALANCE = u64;

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
    InvalidShadowAllocation(Vec<u8>),
    UnableToGetContractBalance(CONTRACT_ID, Option<sled::Error>),
    InvalidBalanceBytesError(Vec<u8>),
    //
    AllocsSumMismatch(CONTRACT_ID, ALLOCS_SUM, ALLOCS_SUM),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, ALLOCS_SUM, CONTRACT_BALANCE),
    ShadowAllocationGetError(CONTRACT_ID, sled::Error),
}

/// The state save error.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderSaveError {
    OpenTreeError(CONTRACT_ID, sled::Error),
    TreeValueInsertError(CONTRACT_ID, ACCOUNT_KEY, u64, sled::Error),
    ContractBodyNotFound(CONTRACT_ID),
}

/// The state register error.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderRegisterError {
    ContractIDAlreadyRegistered(CONTRACT_ID),
    TreeValueInsertError(CONTRACT_ID, sled::Error),
    OpenTreeError(CONTRACT_ID, sled::Error),
}

/// The shadow allocation error.
#[derive(Debug, Clone)]
pub enum ShadowAllocError {
    AccountKeyAlreadyAllocated(CONTRACT_ID, ACCOUNT_KEY),
    ShadowSpaceNotFound(CONTRACT_ID),
}

/// The state save error.
#[derive(Debug, Clone)]
pub enum ShadowAllocUpError {
    ShadowSpaceNotFound(CONTRACT_ID),
    UnableToGetOldAccountAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBalance(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, ALLOCS_SUM, CONTRACT_BALANCE),
}

/// The shadow allocation decrease error.
#[derive(Debug, Clone)]
pub enum ShadowAllocDownError {
    ShadowSpaceNotFound(CONTRACT_ID),
    UnableToGetOldAccountAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBalance(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, ALLOCS_SUM, CONTRACT_BALANCE),
    AllocValueWouldGoBelowZero(CONTRACT_ID, ACCOUNT_KEY, u64, u64),
}
