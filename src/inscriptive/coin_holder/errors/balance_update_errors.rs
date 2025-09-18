/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Errors associated with increasing the account's balance.
#[derive(Debug, Clone)]
pub enum CHAccountBalanceUpError {
    UnableToGetAccountBalance(ACCOUNT_KEY),
}

/// Errors associated with decreasing the account's balance.
#[derive(Debug, Clone)]
pub enum CHAccountBalanceDownError {
    UnableToGetAccountBalance(ACCOUNT_KEY),
    AccountBalanceWouldGoBelowZero(ACCOUNT_KEY, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
}

/// Errors associated with increasing contract's balance.
#[derive(Debug, Clone)]
pub enum CHContractBalanceUpError {
    UnableToGetContractBalance(CONTRACT_ID),
    UnableToGetContractBody(CONTRACT_ID),
}

/// Errors associated with decreasing contract's balance.
#[derive(Debug, Clone)]
pub enum CHContractBalanceDownError {
    UnableToGetContractBalance(CONTRACT_ID),
    ContractBalanceWouldGoBelowZero(CONTRACT_ID, SATOSHI_AMOUNT, SATOSHI_AMOUNT),
    UnableToGetContractBody(CONTRACT_ID),
}
