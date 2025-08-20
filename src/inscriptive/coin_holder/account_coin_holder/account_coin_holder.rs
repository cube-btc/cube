use crate::{
    constructive::entity::account::account::Account,
    inscriptive::coin_holder::account_coin_holder::account_coin_holder_error::AccountCoinHolderConstructionError,
    operative::Chain,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded account coin holder.
#[allow(non_camel_case_types)]
pub type ACCOUNT_COIN_HOLDER = Arc<Mutex<AccountCoinHolder>>;

/// Registery index of an account for efficient referencing (from 1 to U32::MAX).
#[allow(non_camel_case_types)]
type REGISTERY_INDEX = u32;

/// BTC balance of an account in satoshis.
#[allow(non_camel_case_types)]
type COIN_BALANCE = u64;

/// A database manager for storing and retrieving account coin balances.
pub struct AccountCoinHolder {
    // In-memory list of coin balances by registery index.
    coin_balances: HashMap<REGISTERY_INDEX, COIN_BALANCE>,
    // In-storage db for storing the coin balances.
    coin_balances_db: sled::Db,
}

impl AccountCoinHolder {
    /// Construct the account coin holder.
    pub fn new(chain: Chain) -> Result<ACCOUNT_COIN_HOLDER, AccountCoinHolderConstructionError> {
        // Open the account coin holder db.
        let coin_balances_db = {
            let path = format!("{}/{}/{}", "db", chain.to_string(), "coin_holder/account");
            sled::open(path).map_err(AccountCoinHolderConstructionError::DBOpenError)?
        };

        // Initialize the in-memory list of coin balances.
        let mut coin_balances = HashMap::<REGISTERY_INDEX, COIN_BALANCE>::new();

        // Collect the in-memory list of coin balances by registery index.
        for (index, lookup) in coin_balances_db.iter().enumerate() {
            if let Ok((key, val)) = lookup {
                // Key is the 4-byte registery index.
                let registery_index: u32 =
                    u32::from_le_bytes(key.as_ref().try_into().map_err(|_| {
                        AccountCoinHolderConstructionError::RegisteryIndexDeserializeErrorAtIndex(
                            index,
                        )
                    })?);

                // Value is the coin balance.
                let coin_balance: COIN_BALANCE =
                    u64::from_le_bytes(val.as_ref().try_into().map_err(|_| {
                        AccountCoinHolderConstructionError::CoinBalanceDeserializeErrorAtIndex(
                            index,
                        )
                    })?);

                // Insert into the in-memory list of coin balances.
                coin_balances.insert(registery_index, coin_balance);
            }
        }

        // Construct the account coin holder.
        let account_coin_holder = Self {
            coin_balances,
            coin_balances_db,
        };

        // Construct the guarded account coin holder.
        let guarded_account_coin_holder = Arc::new(Mutex::new(account_coin_holder));

        // Return the guarded account coin holder.
        Ok(guarded_account_coin_holder)
    }

    /// Get the coin balance of an account by its account key.
    pub async fn get_coin_balance(&self, account: &Account) -> Option<u64> {
        // Get the registery index of the account.
        let registery_index = account.registery_index()?;

        // Get the coin balance by the registery index.
        self.coin_balances.get(&registery_index).cloned()
    }

    /// Update the coin balance of an account by its account key.
    pub async fn update_coin_balance(
        &mut self,
        account: &Account,
        new_coin_balance: u64,
    ) -> Option<()> {
        // Get the registery index of the account.
        let registery_index = account.registery_index()?;

        // Update the in-memory coin balance of the account by the given registery index.
        self.coin_balances.insert(registery_index, new_coin_balance);

        // Update the in-storage coin balance of the account by the given registery index.
        self.coin_balances_db
            .insert(
                &registery_index.to_le_bytes(),
                &new_coin_balance.to_le_bytes(),
            )
            .ok()?;

        // Return the result.
        Some(())
    }
}
