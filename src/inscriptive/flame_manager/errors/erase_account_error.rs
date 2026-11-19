/// Account Key.
type AccountKey = [u8; 32];

/// Errors associated with erasing an account.
#[derive(Debug, Clone)]
pub enum FMEraseAccountError {
    AccountIsNotRegistered(AccountKey),
    AccountHasJustBeenEphemerallyErased(AccountKey),
}
