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

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// A custom, high-precision satoshi amount.
/// 1 satoshi = 100,000,000 sati-satoshis.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

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
    InvalidShadowAllocValueBytes(Vec<u8>),
    InvalidShadowAllocsSumBytes(Vec<u8>),
    //InvalidShadowAllocation(Vec<u8>),
    UnableToGetContractBalance(CONTRACT_ID, Option<sled::Error>),
    InvalidBalanceBytesError(Vec<u8>),
    //
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, ALLOCS_SUM, CONTRACT_BALANCE),
    ShadowAllocationGetError(CONTRACT_ID, sled::Error),
}

/// The state save error.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderSaveError {
    OpenTreeError(CONTRACT_ID, sled::Error),
    BalanceValueInsertError(CONTRACT_ID, SATOSHI_AMOUNT, sled::Error),
    ShadowSpaceTreeAllocInsertError(CONTRACT_ID, ACCOUNT_KEY, SATI_SATOSHI_AMOUNT, sled::Error),
    ShadowSpaceTreeAllocsSumInsertError(CONTRACT_ID, SATOSHI_AMOUNT, sled::Error),
    ContractBodyNotFound(CONTRACT_ID),
}

/// The state register error.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderRegisterError {
    ContractIDAlreadyRegistered(CONTRACT_ID),
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
    AllocValueWouldGoBelowZero(
        CONTRACT_ID,
        ACCOUNT_KEY,
        SATI_SATOSHI_AMOUNT,
        SATI_SATOSHI_AMOUNT,
    ),
}

/// The shadow allocation increase error.
#[derive(Debug, Clone)]
pub enum ShadowAllocUpAllError {
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractAllocsSum(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, ALLOCS_SUM, CONTRACT_BALANCE),
    ShadowSpaceNotFound(CONTRACT_ID),
    OperationNotPossibleWithZeroAllocsSum(CONTRACT_ID),
}

/// The shadow allocation decrease error.

#[derive(Debug, Clone)]
pub enum ShadowAllocDownAllError {
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractAllocsSum(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, ALLOCS_SUM, CONTRACT_BALANCE),
    ShadowSpaceNotFound(CONTRACT_ID),
    OperationNotPossibleWithZeroAllocsSum(CONTRACT_ID),
    AllocsSumWouldGoBelowZero(CONTRACT_ID, ALLOCS_SUM, SATOSHI_AMOUNT),
    IndividualAllocationWouldGoBelowZero(
        CONTRACT_ID,
        ACCOUNT_KEY,
        SATI_SATOSHI_AMOUNT,
        SATI_SATOSHI_AMOUNT,
    ),
}

/// The contract balance increase error.
#[derive(Debug, Clone)]
pub enum ContractBalanceUpError {
    UnableToGetContractBalance(CONTRACT_ID),
    ContractBodyNotFound(CONTRACT_ID),
}

/// The contract balance decrease error.
#[derive(Debug, Clone)]
pub enum ContractBalanceDownError {
    UnableToGetContractBalance(CONTRACT_ID),
    ContractBodyNotFound(CONTRACT_ID),
    ContractBalanceWouldGoBelowZero(CONTRACT_ID, CONTRACT_BALANCE, SATOSHI_AMOUNT),
}
