/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with registering a new contract.
#[derive(Debug, Clone)]
pub enum RMRegisterContractError {
    ContractHasJustBeenEphemerallyRegistered(ContractId),
    ContractIsAlreadyPermanentlyRegistered(ContractId),
}
