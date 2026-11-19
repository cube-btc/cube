/// Account key.
type AccountKey = [u8; 32];

/// Errors associated with erasing an account.
#[derive(Debug, Clone)]
pub enum PMEraseAccountError {
    AccountIsNotRegistered(AccountKey),
    AccountHasJustBeenEphemerallyErased(AccountKey),
}
