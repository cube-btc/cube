/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with ephemeral account privilege updates.
#[derive(Debug, Clone)]
pub enum PMUpdateAccountError {
    /// Account is not in permanent in-memory state (unknown key, or only ephemerally registered this batch).
    AccountIsNotPermanentlyRegistered(AccountKey),
}

/// Errors associated with ephemeral contract privilege updates.
#[derive(Debug, Clone)]
pub enum PMUpdateContractError {
    /// Contract is not in permanent in-memory state (unknown id, or only ephemerally registered this batch).
    ContractIsNotPermanentlyRegistered(ContractId),
}
