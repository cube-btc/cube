/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Errors associated with increasing the account's shadow allocs sum.
#[derive(Debug, Clone)]
pub enum CHAccountShadowAllocsSumUpError {
    UnableToGetAccountShadowAllocsSum(ACCOUNT_KEY),
    UnableToGetAccountBody(ACCOUNT_KEY),
}

/// Errors associated with decreasing the account's shadow allocs sum.
#[derive(Debug, Clone)]
pub enum CHAccountShadowAllocsSumDownError {
    UnableToGetAccountShadowAllocsSum(ACCOUNT_KEY),
    AccountShadowAllocsSumWouldGoBelowZero(ACCOUNT_KEY, SATI_SATOSHI_AMOUNT, SATI_SATOSHI_AMOUNT),
    UnableToGetAccountBody(ACCOUNT_KEY),
}

/// Errors associated with increasing an account's shadow allocation value in the contract's shadow space.   
#[derive(Debug, Clone)]
pub enum CHShadowUpError {
    UnableToGetAccountShadowAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractBody(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    AccountShadowAllocsSumUpError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        CHAccountShadowAllocsSumUpError,
    ),
}

/// Errors associated with decreasing an account's shadow allocation value in the contract's shadow space.
#[derive(Debug, Clone)]
pub enum CHShadowDownError {
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
    AccountShadowAllocsSumDownError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        CHAccountShadowAllocsSumDownError,
    ),
}

/// Errors associated with increasing an account's shadow allocation value in the contract's shadow space.
#[derive(Debug, Clone)]
pub enum CHShadowUpAllError {
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractAllocsSum(CONTRACT_ID),
    OperationNotPossibleWithZeroAllocsSum(CONTRACT_ID),
    AllocsSumExceedsTheContractBalance(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    UnableToGetContractBody(CONTRACT_ID),
    AccountShadowAllocsSumUpError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        CHAccountShadowAllocsSumUpError,
    ),
}

/// Errors associated with decreasing an account's shadow allocation value in the contract's shadow space.

#[derive(Debug, Clone)]
pub enum CHShadowDownAllError {
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
    AccountShadowAllocsSumDownError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        CHAccountShadowAllocsSumDownError,
    ),
}
