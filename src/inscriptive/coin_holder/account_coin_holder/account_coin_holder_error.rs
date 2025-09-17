/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// Errors associated with constructing the `AccountCoinHolder` struct.
#[derive(Debug, Clone)]
pub enum AccountCoinHolderConstructionError {
    DBOpenError(sled::Error),
    UnableToDeserializeAccountKeyBytesFromTreeName(Vec<u8>),
    TreeOpenError(ACCOUNT_KEY, sled::Error),
    TreeIterError(usize, sled::Error),
    UnableToDeserializeKeyBytesFromTreeKey(ACCOUNT_KEY, usize, Vec<u8>),
    UnableToDeserializeAccountBalanceFromTreeValue(ACCOUNT_KEY, usize, [u8; 1], Vec<u8>),
    UnableToDeserializeAccountShadowAllocsSumFromTreeValue(ACCOUNT_KEY, usize, [u8; 1], Vec<u8>),
    InvalidTreeKeyEncountered(ACCOUNT_KEY, Vec<u8>),
}

/// Errors associated with registering a new account.
#[derive(Debug, Clone)]
pub enum AccountCoinHolderRegisterError {
    AccountHasJustBeenEphemerallyRegistered(ACCOUNT_KEY),
    AccountIsAlreadyPermanentlyRegistered(ACCOUNT_KEY),
}

/// Errors associated with increasing the account's balance.
#[derive(Debug, Clone)]
pub enum AccountBalanceUpError {
    UnableToGetAccountBalance(ACCOUNT_KEY),
}

/// Errors associated with decreasing the account's balance.
#[derive(Debug, Clone)]
pub enum AccountBalanceDownError {
    UnableToGetAccountBalance(ACCOUNT_KEY),
    AccountBalanceWouldGoBelowZero(ACCOUNT_KEY, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
}

/// Errors associated with increasing the account's shadow allocs sum.
#[derive(Debug, Clone)]
pub enum AccountShadowAllocsSumUpError {
    UnableToGetAccountShadowAllocsSum(ACCOUNT_KEY),
    UnableToGetAccountBody(ACCOUNT_KEY),
}

/// Errors associated with decreasing the account's shadow allocs sum.
#[derive(Debug, Clone)]
pub enum AccountShadowAllocsSumDownError {
    UnableToGetAccountShadowAllocsSum(ACCOUNT_KEY),
    AccountShadowAllocsSumWouldGoBelowZero(ACCOUNT_KEY, SATI_SATOSHI_AMOUNT, SATI_SATOSHI_AMOUNT),
    UnableToGetAccountBody(ACCOUNT_KEY),
}

/// Errors associated with saving the account coin holder.
#[derive(Debug, Clone)]
pub enum AccountCoinHolderApplyChangesError {
    TreeValueInsertError(ACCOUNT_KEY, SATOSHI_AMOUNT, sled::Error),
    UnableToGetAccountBody(ACCOUNT_KEY),
    OpenTreeError(ACCOUNT_KEY, sled::Error),
    AccountBalanceValueOnDiskInsertionError(ACCOUNT_KEY, SATOSHI_AMOUNT, sled::Error),
    AccountShadowAllocsSumValueOnDiskInsertionError(ACCOUNT_KEY, SATI_SATOSHI_AMOUNT, sled::Error),
}
