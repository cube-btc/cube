/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with ephemeral account privilege updates.
#[derive(Debug, Clone)]
pub enum PMUpdateAccountError {
    AccountIsNotRegistered(AccountKey),
}

/// Errors associated with ephemeral contract privilege updates.
#[derive(Debug, Clone)]
pub enum PMUpdateContractError {
    ContractIsNotRegistered(ContractId),
}
