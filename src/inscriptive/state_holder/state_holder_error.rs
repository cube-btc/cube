/// Contract ID: 32-byte unique identifier.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// State key.
#[allow(non_camel_case_types)]
type STATE_KEY = Vec<u8>;

/// State value.
#[allow(non_camel_case_types)]
type STATE_VALUE = Vec<u8>;

/// The state construction error.
#[derive(Debug, Clone)]
pub enum StateHolderConstructionError {
    MainDBOpenError(sled::Error),
    SubDBOpenError(CONTRACT_ID, sled::Error),
    InvalidContractIDBytes(Vec<u8>),
    DBIterCollectInvalidKeyAtIndex(usize),
}

/// The state save error.
#[derive(Debug, Clone)]
pub enum StateHolderSaveError {
    OpenTreeError(CONTRACT_ID, sled::Error),
    TreeValueInsertError(CONTRACT_ID, STATE_KEY, STATE_VALUE, sled::Error),
}
