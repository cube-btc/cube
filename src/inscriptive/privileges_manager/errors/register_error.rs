/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Errors associated with ephemerally registering accounts.
#[derive(Debug, Clone)]
pub enum PMRegisterAccountError {
    AccountHasJustBeenEphemerallyRegistered(AccountKey),
    AccountIsAlreadyPermanentlyRegistered(AccountKey),
}

/// Errors associated with ephemerally registering contracts.
#[derive(Debug, Clone)]
pub enum PMRegisterContractError {
    ContractHasJustBeenEphemerallyRegistered(ContractId),
    ContractIsAlreadyPermanentlyRegistered(ContractId),
}
