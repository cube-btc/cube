/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Errors associated with allocating a new account to the contract's shadow space.
#[derive(Debug, Clone)]
pub enum CMContractShadowAllocAccountError {
    AccountHasJustBeenEphemerallyAllocated(CONTRACT_ID, ACCOUNT_KEY),
    AccountHasJustBeenEphemerallyDeallocated(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetAccountAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBody(CONTRACT_ID),
}

/// Errors associated with deallocating an account from the contract's shadow space.
#[derive(Debug, Clone)]
pub enum CMContractShadowDeallocAccountError {
    AccountHasJustBeenEphemerallyAllocated(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetAccountAllocValue(CONTRACT_ID, ACCOUNT_KEY),
    AllocValueIsNonZero(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetEpheremalDeallocList(CONTRACT_ID),
    AccountHasJustBeenEphemerallyDeallocated(CONTRACT_ID, ACCOUNT_KEY),
    UnableToGetContractBody(CONTRACT_ID),
}
