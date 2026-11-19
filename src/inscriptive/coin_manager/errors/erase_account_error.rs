/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Errors associated with erasing an account.
#[derive(Debug, Clone)]
pub enum CMEraseAccountError {
    AccountIsNotRegistered(ACCOUNT_KEY),
    AccountHasJustBeenEphemerallyErased(ACCOUNT_KEY),
}
