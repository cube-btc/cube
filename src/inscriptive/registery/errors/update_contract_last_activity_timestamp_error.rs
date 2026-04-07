/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with updating the last activity timestamp of a contract.
#[derive(Debug, Clone)]
pub enum RMUpdateContractLastActivityTimestampError {
    ContractIsNotRegistered(ContractId),
}
