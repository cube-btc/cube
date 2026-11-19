/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Errors associated with erasing a contract.
#[derive(Debug, Clone)]
pub enum CMEraseContractError {
    ContractIsNotRegistered(CONTRACT_ID),
    ContractHasJustBeenEphemerallyErased(CONTRACT_ID),
}
