/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with constructing the `StateManager` struct.
#[derive(Debug, Clone)]
pub enum SMConstructionError {
    DBOpenError(sled::Error),
    TreeOpenError(ContractId, sled::Error),
    TreeIterError(ContractId, sled::Error),
}
