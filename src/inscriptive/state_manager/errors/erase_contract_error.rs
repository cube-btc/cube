/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with erasing a contract.
#[derive(Debug, Clone)]
pub enum SMEraseContractError {
    ContractIsNotRegistered(ContractId),
    ContractHasJustBeenEphemerallyErased(ContractId),
}
