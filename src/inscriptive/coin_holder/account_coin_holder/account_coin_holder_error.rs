/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Account balance.
#[allow(non_camel_case_types)]
type ACCOUNT_BALANCE = u64;

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// The account coin holder construction error.
#[derive(Debug, Clone)]
pub enum AccountCoinHolderConstructionError {
    BalancesDBOpenError(sled::Error),
    AccountBalanceIterError(sled::Error),
    InvalidAccountKeyBytes(Vec<u8>),
    InvalidAccountBalance(Vec<u8>),
}

/// The account coin holder save error.
#[derive(Debug, Clone)]
pub enum AccountCoinHolderSaveError {
    TreeValueInsertError(ACCOUNT_KEY, ACCOUNT_BALANCE, sled::Error),
}

/// The account coin holder register error.
#[derive(Debug, Clone)]
pub enum AccountCoinHolderRegisterError {
    AccountAlreadyRegistered(ACCOUNT_KEY),
}

/// The account balance increase error.
#[derive(Debug, Clone)]
pub enum AccountBalanceUpError {
    UnableToGetAccountBalance(ACCOUNT_KEY),
}

/// The account balance decrease error.
#[derive(Debug, Clone)]
pub enum AccountBalanceDownError {
    UnableToGetAccountBalance(ACCOUNT_KEY),
    AccountBalanceWouldGoBelowZero(ACCOUNT_KEY, ACCOUNT_BALANCE, SATOSHI_AMOUNT),
}
