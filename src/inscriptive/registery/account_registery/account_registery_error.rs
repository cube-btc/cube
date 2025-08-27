/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// The account registry construction error.
#[derive(Debug, Clone)]
pub enum AccountRegisteryConstructionError {
    AccountsDBOpenError(sled::Error),
    InvalidAccountKeyBytes(Vec<u8>),
    AccountRegisteryTreeOpenError(ACCOUNT_KEY, sled::Error),
    InvalidRegisteryIndexBytes(Vec<u8>),
    InvalidCallCounterBytes(Vec<u8>),
    InvalidKeyByte(Vec<u8>),
}

/// The account registery register error.
#[derive(Debug, Clone)]
pub enum AccountRegisteryRegisterError {
    AccountAlreadyPermanentlyRegistered(ACCOUNT_KEY),
    AccountAlreadyEphemerallyRegistered(ACCOUNT_KEY),
}

/// The account registery increment call counter error.
#[derive(Debug, Clone)]
pub enum AccountRegisteryIncrementCallCounterError {
    AccountNotRegistered(ACCOUNT_KEY),
}

/// The account registery save all error.
#[derive(Debug, Clone)]
pub enum AccountRegisterySaveAllError {
    UnableToGetAccountCallCounter(ACCOUNT_KEY),
    UnableToOpenAccountTree(ACCOUNT_KEY, sled::Error),
    UnableToInsertCallCounter(ACCOUNT_KEY, sled::Error),
}
