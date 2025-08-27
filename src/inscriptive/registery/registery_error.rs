use super::account_registery::account_registery_error::AccountRegisteryConstructionError;
use super::contract_registery::contract_registery_error::ContractRegisteryConstructionError;

/// The registery construction error.
#[derive(Debug, Clone)]
pub enum RegisteryConstructionError {
    AccountRegisteryConstructionError(AccountRegisteryConstructionError),
    ContractRegisteryConstructionError(ContractRegisteryConstructionError),
}
