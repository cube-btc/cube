/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with incrementing the call counter of a contract.
#[derive(Debug, Clone)]
pub enum RMIncrementContractCallCounterError {
    ContractIsNotRegistered(ContractId),
}
