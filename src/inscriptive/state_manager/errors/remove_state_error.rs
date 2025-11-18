/// Errors associated with removing a state.
/// Contract ID.
type ContractId = [u8; 32];

/// State key.
type StateKey = Vec<u8>;

/// Errors associated with removing a state.
#[derive(Debug, Clone)]
pub enum SMRemoveStateError {
    ContractNotRegistered(ContractId),
    StateDoesNotExist(ContractId, StateKey),
}
