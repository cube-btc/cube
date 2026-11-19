/// Account Key.
type AccountKey = [u8; 32];

/// Errors associated with erasing an account.
#[derive(Debug, Clone)]
pub enum RMEraseAccountError {
    AccountIsNotRegistered(AccountKey),
    AccountHasJustBeenEphemerallyErased(AccountKey),
}
