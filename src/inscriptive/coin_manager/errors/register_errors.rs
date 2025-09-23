/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Errors associated with registering a new account.
#[derive(Debug, Clone)]
pub enum CMRegisterAccountError {
    AccountHasJustBeenEphemerallyRegistered(ACCOUNT_KEY),
    AccountIsAlreadyPermanentlyRegistered(ACCOUNT_KEY),
}

/// Errors associated with registering a new contract.
#[derive(Debug, Clone)]
pub enum CMRegisterContractError {
    ContractHasJustBeenEphemerallyRegistered(CONTRACT_ID),
    ContractIsAlreadyPermanentlyRegistered(CONTRACT_ID),
}
