/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with erasing a contract.
#[derive(Debug, Clone)]
pub enum RMEraseContractError {
    ContractIsNotRegistered(ContractId),
    ContractHasJustBeenEphemerallyErased(ContractId),
}
