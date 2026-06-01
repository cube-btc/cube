/// Account key.
type AccountKey = [u8; 32];

/// Errors associated with burying an account.
#[derive(Debug, Clone)]
pub enum GraveyardBuryAccountError {
    AccountHasJustBeenEpheremallyBuried(AccountKey),
    AccountIsAlreadyPermanentlyBuried(AccountKey),
}
