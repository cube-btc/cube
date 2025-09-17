use super::account_coin_holder::account_coin_holder::{AccountCoinHolder, ACCOUNT_COIN_HOLDER};
use super::coin_holder_error::CoinHolderConstructionError;
use super::contract_coin_holder::contract_coin_holder::{ContractCoinHolder, CONTRACT_COIN_HOLDER};
use crate::operative::Chain;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A struct for containing both account and contract coin holders.
pub struct CoinHolder {
    /// Account coin holder for managing account balances.
    account_coin_holder: ACCOUNT_COIN_HOLDER,
    /// Contract coin holder for managing contract balances and shadow spaces.
    contract_coin_holder: CONTRACT_COIN_HOLDER,
}

/// Guarded coin holder.
#[allow(non_camel_case_types)]
pub type COIN_HOLDER = Arc<Mutex<CoinHolder>>;

impl CoinHolder {
    /// Initialize the coin holder for the given chain.
    pub fn new(chain: Chain) -> Result<COIN_HOLDER, CoinHolderConstructionError> {
        // Initialize the account coin holder.
        let account_coin_holder = AccountCoinHolder::new(chain)
            .map_err(CoinHolderConstructionError::AccountCoinHolderConstructionError)?;

        // Initialize the contract coin holder.
        let contract_coin_holder = ContractCoinHolder::new(chain, &account_coin_holder)
            .map_err(CoinHolderConstructionError::ContractCoinHolderConstructionError)?;

        // Create the coin holder.
        let coin_holder = CoinHolder {
            account_coin_holder,
            contract_coin_holder,
        };

        // Guard the coin holder.
        let guarded_coin_holder = Arc::new(Mutex::new(coin_holder));

        // Return the guarded coin holder.
        Ok(guarded_coin_holder)
    }

    /// Get a reference to the account coin holder.
    pub fn account_coin_holder(&self) -> ACCOUNT_COIN_HOLDER {
        Arc::clone(&self.account_coin_holder)
    }

    /// Get a reference to the contract coin holder.
    pub fn contract_coin_holder(&self) -> CONTRACT_COIN_HOLDER {
        Arc::clone(&self.contract_coin_holder)
    }
}

/// Erase coin holder data by chain.
pub fn erase_coin_holder(chain: Chain) {
    // Erase account coin holder data.
    super::account_coin_holder::account_coin_holder::erase_account_coin_holder(chain);

    // Erase contract coin holder data.
    super::contract_coin_holder::contract_coin_holder::erase_contract_coin_holder(chain);
}
