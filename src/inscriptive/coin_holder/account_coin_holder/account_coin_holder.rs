use super::account_coin_holder_error::{
    AccountBalanceDownError, AccountCoinHolderConstructionError, AccountCoinHolderSaveError,
};
use crate::inscriptive::coin_holder::account_coin_holder::account_coin_holder_error::{
    AccountBalanceUpError, AccountCoinHolderRegisterError,
};
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// BTC balance of an account in satoshis.
#[allow(non_camel_case_types)]
type ACCOUNT_COIN_BALANCE = u64;

/// A database manager for handling account balances.
/// For now, we are caching everything in memory.
pub struct AccountCoinHolder {
    /// In-memory cache of account balances: ACCOUNT_KEY -> ACCOUNT_COIN_BALANCE
    in_memory: HashMap<ACCOUNT_KEY, ACCOUNT_COIN_BALANCE>,
    /// Sled DB with all account balances in a single tree.
    balance_db: sled::Db,
    /// In-memory cache of ephemeral account balances.
    ephemeral_coins: HashMap<ACCOUNT_KEY, ACCOUNT_COIN_BALANCE>,
    /// In-memory cache of ephemeral account balances backup.
    ephemeral_coins_backup: HashMap<ACCOUNT_KEY, ACCOUNT_COIN_BALANCE>,
}

/// Guarded account coin holder.
#[allow(non_camel_case_types)]
pub type ACCOUNT_COIN_HOLDER = Arc<Mutex<AccountCoinHolder>>;

impl AccountCoinHolder {
    /// Initialize the state for the given chain
    pub fn new(chain: Chain) -> Result<ACCOUNT_COIN_HOLDER, AccountCoinHolderConstructionError> {
        // Open the balance db.
        let balance_path = format!("db/{}/coin/account/balance", chain.to_string());
        let balance_db = sled::open(balance_path)
            .map_err(AccountCoinHolderConstructionError::BalancesDBOpenError)?;

        // Initialize the in-memory cache of account coins.
        let mut in_memory = HashMap::<ACCOUNT_KEY, ACCOUNT_COIN_BALANCE>::new();

        // Iterate over all account balances in the balance db.
        for account_balance in balance_db.iter() {
            // Get the key and value.
            let (k, v) = match account_balance {
                Ok((k, v)) => (k, v),
                Err(e) => {
                    return Err(AccountCoinHolderConstructionError::AccountBalanceIterError(
                        e,
                    ));
                }
            };

            // Convert the key to an account key.
            let account_key: [u8; 32] = k.to_vec().try_into().map_err(|_| {
                AccountCoinHolderConstructionError::InvalidAccountKeyBytes(k.to_vec())
            })?;

            // Convert the value to an account balance.
            let account_balance: u64 = u64::from_le_bytes(v.as_ref().try_into().map_err(|_| {
                AccountCoinHolderConstructionError::InvalidAccountBalance(v.to_vec())
            })?);

            // Insert the account balance into the in-memory cache.
            in_memory.insert(account_key, account_balance);
        }

        // Create the state holder.
        let account_coin_holder = AccountCoinHolder {
            in_memory,
            balance_db,
            ephemeral_coins: HashMap::<ACCOUNT_KEY, ACCOUNT_COIN_BALANCE>::new(),
            ephemeral_coins_backup: HashMap::<ACCOUNT_KEY, ACCOUNT_COIN_BALANCE>::new(),
        };

        // Return the guarded state holder.
        Ok(Arc::new(Mutex::new(account_coin_holder)))
    }

    /// Clones ephemeral states into the backup.
    fn backup_ephemeral_states(&mut self) {
        self.ephemeral_coins_backup = self.ephemeral_coins.clone();
    }

    /// Restores ephemeral states from the backup.
    fn restore_ephemeral_states(&mut self) {
        self.ephemeral_coins = self.ephemeral_coins_backup.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the ephemeral states.
        self.backup_ephemeral_states();
    }

    /// Get the account coin balance for an account key.
    pub fn get_account_balance(&self, account_key: ACCOUNT_KEY) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some(balance) = self.ephemeral_coins.get(&account_key) {
            return Some(*balance);
        }

        // And then try to get from the permanent in-memory states.
        self.in_memory.get(&account_key).cloned()
    }

    /// Registers an account if it is not already registered.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn register_account(
        &mut self,
        account_key: ACCOUNT_KEY,
    ) -> Result<(), AccountCoinHolderRegisterError> {
        // Check if the account is already registered by retrieving its balance.
        if self.get_account_balance(account_key).is_some() {
            return Err(AccountCoinHolderRegisterError::AccountAlreadyRegistered(
                account_key,
            ));
        }

        // Insert into the ephemeral states with zero balance.
        self.ephemeral_coins.insert(account_key, 0);

        // Return the result.
        Ok(())
    }

    /// Increases an account balance by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn account_balance_up(
        &mut self,
        account_key: ACCOUNT_KEY,
        up_value_in_satoshis: u64,
    ) -> Result<(), AccountBalanceUpError> {
        // Try to get the  account balance.
        let existing_account_balance_in_satoshis: u64 = match self.get_account_balance(account_key)
        {
            // If the account is already registered, return the balance.
            Some(balance) => balance,
            // If the account is not registered, return an error.
            None => {
                return Err(AccountBalanceUpError::UnableToGetAccountBalance(
                    account_key,
                ))
            }
        };

        // Calculate the new account balance.
        let new_account_balance_in_satoshis: u64 =
            existing_account_balance_in_satoshis + up_value_in_satoshis;

        // Insert (or update) the balance into the ephemeral states.
        self.ephemeral_coins
            .insert(account_key, new_account_balance_in_satoshis);

        // Return the result.
        Ok(())
    }

    /// Decreases an account balance by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn account_balance_down(
        &mut self,
        account_key: ACCOUNT_KEY,
        down_value_in_satoshis: u64,
    ) -> Result<(), AccountBalanceDownError> {
        // Try to get the account balance.
        let existing_account_balance_in_satoshis: u64 = match self.get_account_balance(account_key)
        {
            // If the account is already registered, return the balance.
            Some(balance) => balance,
            // If the account is not registered, return an error.
            None => {
                return Err(AccountBalanceDownError::UnableToGetAccountBalance(
                    account_key,
                ))
            }
        };

        // Check if the decrease would make the account balance go below zero.
        if down_value_in_satoshis > existing_account_balance_in_satoshis {
            return Err(AccountBalanceDownError::AccountBalanceWouldGoBelowZero(
                account_key,
                existing_account_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new account balance.
        let new_account_balance_in_satoshis: u64 =
            existing_account_balance_in_satoshis - down_value_in_satoshis;

        // Insert (or update) the balance into the ephemeral states.
        self.ephemeral_coins
            .insert(account_key, new_account_balance_in_satoshis);

        // Return the result.
        Ok(())
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        // Restore the ephemeral states from the backup.
        self.restore_ephemeral_states();
    }

    /// Clears all epheremal changes.
    pub fn rollback_all(&mut self) {
        // Clear the ephemeral states.
        self.ephemeral_coins.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_coins_backup.clear();
    }

    /// Saves all epheremal changes in-memory and on-disk.
    pub fn save_all(&mut self) -> Result<(), AccountCoinHolderSaveError> {
        // Iterate over all ephemeral states.
        for (account_key, ephemeral_balance) in self.ephemeral_coins.iter() {
            // In-memory insertion.
            {
                // Insert or update the balance in memory.
                self.in_memory.insert(*account_key, *ephemeral_balance);
            }

            // On-disk insertion.
            {
                // Save the balance to the balance db.
                self.balance_db
                    .insert(account_key, ephemeral_balance.to_le_bytes().to_vec())
                    .map_err(|e| {
                        AccountCoinHolderSaveError::TreeValueInsertError(
                            account_key.to_owned(),
                            *ephemeral_balance,
                            e,
                        )
                    })?;
            }
        }

        // Clear the ephemeral states.
        self.ephemeral_coins.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_coins_backup.clear();

        Ok(())
    }
}

/// Erase by db path.
pub fn erase_account_coin_holder(chain: Chain) {
    // Balance db path.
    let balance_path = format!("db/{}/coin/account/balance", chain.to_string());

    // Erase the path.
    let _ = std::fs::remove_dir_all(balance_path);
}
