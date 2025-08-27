/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// The contract registry construction error.
#[derive(Debug, Clone)]
pub enum ContractRegisteryConstructionError {
    ContractsDBOpenError(sled::Error),
    InvalidContractIDBytes(Vec<u8>),
    ContractRegisteryTreeOpenError(CONTRACT_ID, sled::Error),
    InvalidRegisteryIndexBytes(Vec<u8>),
    InvalidCallCounterBytes(Vec<u8>),
    InvalidKeyByte(Vec<u8>),
}

/// The contract registry registration error.
#[derive(Debug, Clone)]
pub enum ContractRegisteryRegisterError {
    ContractAlreadyPermanentlyRegistered(CONTRACT_ID),
    ContractAlreadyEphemerallyRegistered(CONTRACT_ID),
}

/// The contract registry increment call counter error.
#[derive(Debug, Clone)]
pub enum ContractRegisteryIncrementCallCounterError {
    ContractNotRegistered(CONTRACT_ID),
}

/// The contract registry save all error.
#[derive(Debug, Clone)]
pub enum ContractRegisterySaveAllError {
    UnableToGetContractCallCounter(CONTRACT_ID),
    UnableToOpenContractTree(CONTRACT_ID, sled::Error),
    UnableToInsertCallCounter(CONTRACT_ID, sled::Error),
}
