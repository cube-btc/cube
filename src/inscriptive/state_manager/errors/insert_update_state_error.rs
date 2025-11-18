/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with inserting or updating a state.
#[derive(Debug, Clone)]
pub enum SMInsertUpdateStateError {
    ContractNotRegistered(ContractId),
}
