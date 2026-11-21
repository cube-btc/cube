use crate::inscriptive::coin_manager::errors::balance_update_errors::CMContractVitalityBalanceDownError;
use crate::inscriptive::coin_manager::errors::erase_account_error::CMEraseAccountError;
use crate::inscriptive::coin_manager::errors::erase_contract_error::CMEraseContractError;
use crate::inscriptive::flame_manager::errors::erase_account_error::FMEraseAccountError;
use crate::inscriptive::graveyard::errors::bury_account_error::GraveyardBuryAccountError;
use crate::inscriptive::graveyard::errors::bury_contract_error::GraveyardBuryContractError;
use crate::inscriptive::privileges_manager::errors::erase_account_error::PMEraseAccountError;
use crate::inscriptive::privileges_manager::errors::erase_contract_error::PMEraseContractError;
use crate::inscriptive::privileges_manager::errors::update_error::{
    PMUpdateAccountError, PMUpdateContractError,
};
use crate::inscriptive::registry::errors::erase_account_error::RMEraseAccountError;
use crate::inscriptive::registry::errors::erase_contract_error::RMEraseContractError;
use crate::inscriptive::state_manager::errors::erase_contract_error::SMEraseContractError;

/// Account key.
type AccountKey = [u8; 32];

/// Contract id.
type ContractId = [u8; 32];

/// Errors associated with periodic termination.
#[derive(Debug, Clone)]
pub enum TerminationError {
    /// Unable to get the termination fees.
    UnableToGetTerminationFees,

    /// Unable to update an account liveness flag.
    UnableToUpdateAccountLivenessFlag(AccountKey, PMUpdateAccountError),

    /// Unable to update a contract liveness flag.
    UnableToUpdateContractLivenessFlag(ContractId, PMUpdateContractError),

    /// Unable to apply occupancy tax by decreasing contract vitality.
    UnableToApplyContractVitalityTax(ContractId, CMContractVitalityBalanceDownError),

    /// Unable to bury an account in the graveyard.
    UnableToBuryAccount(AccountKey, GraveyardBuryAccountError),

    /// Unable to bury a contract in the graveyard.
    UnableToBuryContract(ContractId, GraveyardBuryContractError),

    /// Unable to erase an account from the coin manager.
    UnableToEraseAccountFromCoinManager(AccountKey, CMEraseAccountError),

    /// Unable to erase a contract from the coin manager.
    UnableToEraseContractFromCoinManager(ContractId, CMEraseContractError),

    /// Unable to erase an account from the flame manager.
    UnableToEraseAccountFromFlameManager(AccountKey, FMEraseAccountError),

    /// Unable to erase an account from the privileges manager.
    UnableToEraseAccountFromPrivilegesManager(AccountKey, PMEraseAccountError),

    /// Unable to erase a contract from the privileges manager.
    UnableToEraseContractFromPrivilegesManager(ContractId, PMEraseContractError),

    /// Unable to erase an account from the registry.
    UnableToEraseAccountFromRegistry(AccountKey, RMEraseAccountError),

    /// Unable to erase a contract from the registry.
    UnableToEraseContractFromRegistry(ContractId, RMEraseContractError),

    /// Unable to erase a contract from the state manager.
    UnableToEraseContractFromStateManager(ContractId, SMEraseContractError),
}
