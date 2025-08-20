/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Account balance.
#[allow(non_camel_case_types)]
type ACCOUNT_BALANCE = u64;

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
