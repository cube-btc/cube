use crate::inscriptive::coin_holder::account_coin_holder::account_coin_holder_error::{
    AccountShadowAllocsSumDownError, AccountShadowAllocsSumUpError,
};

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// State key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// A custom, high-precision satoshi amount.
/// 1 satoshi = 100,000,000 sati-satoshis.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// Errors associated with constructing the `ContractCoinHolder` struct.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderConstructionError {
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

/// Errors associated with registering a new contract.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderRegisterContractError {
    ContractHasJustBeenEphemerallyRegistered(CONTRACT_ID),
    ContractIsAlreadyPermanentlyRegistered(CONTRACT_ID),
}

/// Errors associated with allocating a new account to the contract's shadow space.
#[derive(Debug, Clone)]
pub enum ShadowAllocAccountError {
    AccountHasJustBeenEphemerallyAllocated(CONTRACT_ID, ACCOUNT_KEY),
    AccountHasJustBeenEphemerallyDeallocated(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetAccountAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBody(CONTRACT_ID),
}

/// Errors associated with deallocating an account from the contract's shadow space.
#[derive(Debug, Clone)]
pub enum ShadowDeallocAccountError {
    AccountHasJustBeenEphemerallyAllocated(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetAccountAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    AllocValueIsNonZero(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetEpheremalDeallocList(CONTRACT_ID),
    AccountHasJustBeenEphemerallyDeallocated(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBody(CONTRACT_ID),
}

/// Errors associated with increasing contract's balance.
#[derive(Debug, Clone)]
pub enum ContractBalanceUpError {
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractBody(CONTRACT_ID),
}

/// Errors associated with decreasing contract's balance.
#[derive(Debug, Clone)]
pub enum ContractBalanceDownError {
    UnableToGetContractBalance(CONTRACT_ID),
    ContractBalanceWouldGoBelowZero(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    UnableToGetContractBody(CONTRACT_ID),
}

/// Errors associated with increasing an account's shadow allocation value in the contract's shadow space.   
#[derive(Debug, Clone)]
pub enum ShadowAllocUpError {
    UnableToGetAccountShadowAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractBody(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    AccountCoinHolderShadowAllocsSumUpError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        AccountShadowAllocsSumUpError,
    ),
}

/// Errors associated with decreasing an account's shadow allocation value in the contract's shadow space.
#[derive(Debug, Clone)]
pub enum ShadowAllocDownError {
    UnableToGetAccountShadowAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBalance(CONTRACT_ID),
    AccountShadowAllocValueWouldGoBelowZero(
        CONTRACT_ID,
        ACCOUNT_KEY,
        SATI_SATOSHI_AMOUNT,
        SATI_SATOSHI_AMOUNT,
    ),
    UnableToGetContractBody(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    AccountCoinHolderShadowAllocsSumDownError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        AccountShadowAllocsSumDownError,
    ),
}

/// Errors associated with increasing an account's shadow allocation value in the contract's shadow space.
#[derive(Debug, Clone)]
pub enum ShadowAllocUpAllError {
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractAllocsSum(CONTRACT_ID),
    OperationNotPossibleWithZeroAllocsSum(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    UnableToGetContractBody(CONTRACT_ID),
    AccountCoinHolderShadowAllocsSumUpError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        AccountShadowAllocsSumUpError,
    ),
}

/// Errors associated with decreasing an account's shadow allocation value in the contract's shadow space.

#[derive(Debug, Clone)]
pub enum ShadowAllocDownAllError {
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractAllocsSum(CONTRACT_ID),
    OperationNotPossibleWithZeroAllocsSum(CONTRACT_ID),
    AllocsSumWouldGoBelowZero(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    UnableToGetContractBody(CONTRACT_ID),
    AccountShadowAllocValueWouldGoBelowZero(
        CONTRACT_ID,
        ACCOUNT_KEY,
        SATI_SATOSHI_AMOUNT,
        SATI_SATOSHI_AMOUNT,
    ),
    AccountCoinHolderShadowAllocsSumDownError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        AccountShadowAllocsSumDownError,
    ),
}

/// Errors associated with applying changes to the `ContractCoinHolder`.
#[derive(Debug, Clone)]
pub enum ContractCoinHolderApplyChangesError {
    OpenTreeError(CONTRACT_ID, sled::Error),
    BalanceValueOnDiskInsertionError(CONTRACT_ID, SATOSHI_AMOUNT, sled::Error),
    AllocsSumValueOnDiskInsertionError(CONTRACT_ID, SATOSHI_AMOUNT, sled::Error),
    UnableToGetContractBody(CONTRACT_ID),
    ShadowAllocValueOnDiskInsertionError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        SATI_SATOSHI_AMOUNT,
        sled::Error,
    ),
    InMemoryDeallocAccountError(CONTRACT_ID, ACCOUNT_KEY),
    OnDiskDeallocAccountError(CONTRACT_ID, ACCOUNT_KEY, sled::Error),
}
