/// Contract registery index.
#[allow(non_camel_case_types)]
type CONTRACT_REGISTERY_INDEX = u32;

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
    SubDBOpenError(CONTRACT_REGISTERY_INDEX, sled::Error),
    InvalidContractRegisteryIndexBytes(Vec<u8>),
    DBIterCollectInvalidKeyAtIndex(usize),
}

/// The state save error.
#[derive(Debug, Clone)]
pub enum StateHolderSaveError {
    OpenTreeError(CONTRACT_REGISTERY_INDEX, sled::Error),
    TreeValueInsertError(
        CONTRACT_REGISTERY_INDEX,
        STATE_KEY,
        STATE_VALUE,
        sled::Error,
    ),
}
