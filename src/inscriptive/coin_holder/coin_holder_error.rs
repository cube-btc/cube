use super::account_coin_holder::account_coin_holder_error::AccountCoinHolderConstructionError;
use super::contract_coin_holder::contract_coin_holder_error::ContractCoinHolderConstructionError;

/// The coin holder construction error.
#[derive(Debug, Clone)]
pub enum CoinHolderConstructionError {
    AccountCoinHolderConstructionError(AccountCoinHolderConstructionError),
    ContractCoinHolderConstructionError(ContractCoinHolderConstructionError),
}
