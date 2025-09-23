/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Errors associated with constructing the `CoinHolder` struct for accounts.
#[derive(Debug, Clone)]
pub enum CMConstructionAccountError {
    DBOpenError(sled::Error),
    UnableToDeserializeAccountKeyBytesFromTreeName(Vec<u8>),
    TreeOpenError(ACCOUNT_KEY, sled::Error),
    TreeIterError(usize, sled::Error),
    UnableToDeserializeKeyBytesFromTreeKey(ACCOUNT_KEY, usize, Vec<u8>),
    UnableToDeserializeAccountBalanceFromTreeValue(ACCOUNT_KEY, usize, [u8; 1], Vec<u8>),
    UnableToDeserializeAccountShadowAllocsSumFromTreeValue(ACCOUNT_KEY, usize, [u8; 1], Vec<u8>),
    InvalidTreeKeyEncountered(ACCOUNT_KEY, Vec<u8>),
}

/// Errors associated with constructing the `CoinHolder` struct for contracts.
#[derive(Debug, Clone)]
pub enum CMConstructionContractError {
    DBOpenError(sled::Error),
    UnableToDeserializeContractIDBytesFromTreeName(Vec<u8>),
    TreeOpenError(CONTRACT_ID, sled::Error),
    TreeIterError(CONTRACT_ID, usize, sled::Error),
    UnableToDeserializeKeyBytesFromTreeKey(CONTRACT_ID, usize, Vec<u8>),
    UnableToDeserializeContractBalanceFromTreeValue(CONTRACT_ID, usize, [u8; 32], Vec<u8>),
    UnableToDeserializeAllocsSumFromTreeValue(CONTRACT_ID, usize, [u8; 32], Vec<u8>),
    UnableToDeserializeAllocValueFromTreeValue(CONTRACT_ID, usize, [u8; 32], Vec<u8>),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
}

/// Errors associated with constructing the `CoinHolder` struct.
#[derive(Debug, Clone)]
pub enum CMConstructionError {
    AccountConstructionError(CMConstructionAccountError),
    ContractConstructionError(CMConstructionContractError),
}
