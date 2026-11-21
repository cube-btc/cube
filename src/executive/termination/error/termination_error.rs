use crate::inscriptive::coin_manager::errors::erase_account_error::CMEraseAccountError;
use crate::inscriptive::flame_manager::errors::erase_account_error::FMEraseAccountError;
use crate::inscriptive::graveyard::errors::bury_account_error::GraveyardBuryAccountError;
use crate::inscriptive::privileges_manager::errors::erase_account_error::PMEraseAccountError;
use crate::inscriptive::privileges_manager::errors::update_error::PMUpdateAccountError;
use crate::inscriptive::registry::errors::erase_account_error::RMEraseAccountError;

/// Account key.
type AccountKey = [u8; 32];

/// Errors associated with periodic termination.
#[derive(Debug, Clone)]
pub enum TerminationError {
    /// Unable to get the termination fees.
    UnableToGetTerminationFees,

    /// Unable to update an account liveness flag.
    UnableToUpdateAccountLivenessFlag(AccountKey, PMUpdateAccountError),

    /// Unable to bury an account in the graveyard.
    UnableToBuryAccount(AccountKey, GraveyardBuryAccountError),

    /// Unable to erase an account from the coin manager.
    UnableToEraseAccountFromCoinManager(AccountKey, CMEraseAccountError),

    /// Unable to erase an account from the flame manager.
    UnableToEraseAccountFromFlameManager(AccountKey, FMEraseAccountError),

    /// Unable to erase an account from the privileges manager.
    UnableToEraseAccountFromPrivilegesManager(AccountKey, PMEraseAccountError),

    /// Unable to erase an account from the registry.
    UnableToEraseAccountFromRegistry(AccountKey, RMEraseAccountError),
}
