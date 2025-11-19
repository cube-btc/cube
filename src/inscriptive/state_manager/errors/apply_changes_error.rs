/// Contract ID.
type ContractId = [u8; 32];

/// State key.
type StateKey = Vec<u8>;

/// State value.
type StateValue = Vec<u8>;

/// Errors associated with applying changes to the `StateManager`.
#[derive(Debug, Clone)]
pub enum SMApplyChangesError {
    TreeOpenError(ContractId, sled::Error),
    ContractIdNotFoundInMemory(ContractId),
    TreeValueInsertError(ContractId, StateKey, StateValue, sled::Error),
    TreeValueRemoveError(ContractId, StateKey, sled::Error),
}
