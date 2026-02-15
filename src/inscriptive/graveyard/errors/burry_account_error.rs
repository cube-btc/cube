/// Account key.
type AccountKey = [u8; 32];

/// Errors associated with burrying an account.
#[derive(Debug, Clone)]
pub enum GraveyardBurryAccountError {
    AccountHasJustBeenEpheremallyBurried(AccountKey),
    AccountIsAlreadyPermanentlyBurried(AccountKey),
}
