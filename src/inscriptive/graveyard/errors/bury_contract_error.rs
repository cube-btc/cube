/// Contract id.
type ContractId = [u8; 32];

/// Errors associated with burying a contract.
#[derive(Debug, Clone)]
pub enum GraveyardBuryContractError {
    ContractHasJustBeenEphemerallyBuried(ContractId),
    ContractIsAlreadyPermanentlyBuried(ContractId),
}
