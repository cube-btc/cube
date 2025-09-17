use super::account_coin_holder_error::{
    AccountBalanceDownError, AccountCoinHolderApplyChangesError, AccountCoinHolderConstructionError,
};
use crate::inscriptive::coin_holder::account_coin_holder::account_coin_holder_error::{
    AccountBalanceUpError, AccountCoinHolderRegisterError, AccountShadowAllocsSumDownError,
    AccountShadowAllocsSumUpError,
};
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// Special db key for the account balance (0x00..).
const ACCOUNT_BALANCE_SPECIAL_KEY: [u8; 1] = [0x00; 1];

/// Special db key for the account shadow allocs sum (0x01..).
const ACCOUNT_ALLOCS_SUM_SPECIAL_KEY: [u8; 1] = [0x01; 1];

/// A struct for containing account balance and shadow allocs sum of all contracts.
#[derive(Clone)]
struct AccountBody {
    // Account balance.
    pub balance: SATOSHI_AMOUNT,

    // Individual shadow allocs sum of all contracts.
    pub shadow_allocs_sum: SATI_SATOSHI_AMOUNT,
}

/// A struct for containing state differences to be applied.
#[derive(Clone)]
struct Delta {
    // New accounts to register.
    pub new_accounts_to_register: Vec<ACCOUNT_KEY>,

    // Updated account balances for a given account.
    pub updated_account_balances: HashMap<ACCOUNT_KEY, SATOSHI_AMOUNT>,

    // Updated shadow allocs sums for a given account.
    pub updated_shadow_allocs_sums: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
}

/// A database manager for handling account balances.
///
/// NOTE: For now, we are caching *everything* in memory.
pub struct AccountCoinHolder {
    // IN-MEMORY STATES
    in_memory: HashMap<ACCOUNT_KEY, AccountBody>,

    // ON-DISK STATES
    on_disk: sled::Db,

    // STATE DIFFERENCES TO BE APPLIED
    delta: Delta,

    // BACKUP OF STATE DIFFERENCES IN CASE OF ROLLBACK
    delta_backup: Delta,
}

/// Guarded account coin holder.
#[allow(non_camel_case_types)]
pub type ACCOUNT_COIN_HOLDER = Arc<Mutex<AccountCoinHolder>>;

impl AccountCoinHolder {
    /// Initialize the state for the given chain
    pub fn new(chain: Chain) -> Result<ACCOUNT_COIN_HOLDER, AccountCoinHolderConstructionError> {
        // Open the respective database.
        let db_path = format!("db/{}/coin/account", chain.to_string());
        let db = sled::open(db_path).map_err(AccountCoinHolderConstructionError::DBOpenError)?;

        // Initialize the in-memory list.
        let mut in_memory = HashMap::<ACCOUNT_KEY, AccountBody>::new();

        // Iterate over all trees in the database.
        for tree_name in db.tree_names() {
            // Deserialize account key bytes from tree name.
            let account_key: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                AccountCoinHolderConstructionError::UnableToDeserializeAccountKeyBytesFromTreeName(
                    tree_name.to_vec(),
                )
            })?;

            // Open the tree.
            let tree = db
                .open_tree(tree_name)
                .map_err(|e| AccountCoinHolderConstructionError::TreeOpenError(account_key, e))?;

            let mut account_balance: u64 = 0;
            let mut account_shadow_allocs_sum: u128 = 0;

            // iterate over all items in the tree.
            for (index, item) in tree.iter().enumerate() {
                // Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(AccountCoinHolderConstructionError::TreeIterError(index, e));
                    }
                };

                // Deserialize the key bytes.
                let tree_key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    AccountCoinHolderConstructionError::UnableToDeserializeKeyBytesFromTreeKey(
                        account_key,
                        index,
                        key.to_vec(),
                    )
                })?;

                // Match the tree key bytes.
                match tree_key_byte {
                    // If the key is (0x00..), it is a special key that corresponds to the account balance value.
                    ACCOUNT_BALANCE_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let account_balance_deserialized: u64 =
                            u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                AccountCoinHolderConstructionError::UnableToDeserializeAccountBalanceFromTreeValue(
                                    account_key,
                                    index,
                                    tree_key_byte,
                                    value.to_vec(),
                                )
                            })?);

                        // Update the account balance.
                        account_balance = account_balance_deserialized;
                    }
                    // If the key is (0x01..), it is a special key that corresponds to the account shadow allocs sum value.
                    ACCOUNT_ALLOCS_SUM_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let account_shadow_allocs_sum_deserialized: u128 =
                            u128::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                AccountCoinHolderConstructionError::UnableToDeserializeAccountShadowAllocsSumFromTreeValue(
                                    account_key,
                                    index,
                                    tree_key_byte,
                                    value.to_vec(),
                                )
                            })?);

                        // Update the account shadow allocs sum.
                        account_shadow_allocs_sum = account_shadow_allocs_sum_deserialized;
                    }
                    _ => {
                        // This key is a normal account key that corresponds to an account allocation.
                        return Err(
                            AccountCoinHolderConstructionError::InvalidTreeKeyEncountered(
                                account_key,
                                tree_key_byte.to_vec(),
                            ),
                        );
                    }
                }
            }

            // Construct the account body.
            let account_body = AccountBody {
                balance: account_balance,
                shadow_allocs_sum: account_shadow_allocs_sum,
            };

            // Insert the account body into the in-memory list.
            in_memory.insert(account_key, account_body);
        }

        // Create a fresh new delta.
        let fresh_new_delta = Delta {
            new_accounts_to_register: Vec::new(),
            updated_account_balances: HashMap::new(),
            updated_shadow_allocs_sums: HashMap::new(),
        };

        // Create the state holder.
        let account_coin_holder = AccountCoinHolder {
            in_memory,
            on_disk: db,
            delta: fresh_new_delta.clone(),
            delta_backup: fresh_new_delta,
        };

        // Return the guarded state holder.
        Ok(Arc::new(Mutex::new(account_coin_holder)))
    }

    /// Clones ephemeral states into the backup.
    fn backup_delta(&mut self) {
        self.delta_backup = self.delta.clone();
    }

    /// Restores ephemeral states from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.delta_backup.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the delta.
        self.backup_delta();
    }

    /// Checks if an account is registered.
    ///
    /// NOTE: Permanant registrations only. Epheremal in-Delta registrations are out of scope.
    pub fn is_account_registered(&self, account_key: ACCOUNT_KEY) -> bool {
        self.in_memory.contains_key(&account_key)
    }

    /// Returns the account balance for an account key in satoshis.
    pub fn get_account_balance(&self, account_key: ACCOUNT_KEY) -> Option<u64> {
        // Try to get from the delta first.
        if let Some(value) = self.delta.updated_account_balances.get(&account_key) {
            return Some(value.clone());
        }

        // And then try to get from the permanent in-memory states.
        self.in_memory
            .get(&account_key)
            .cloned()
            .map(|account_body| account_body.balance)
    }

    /// Returns the account shadow allocs sum for an account key in sati-satoshis.
    pub fn get_account_shadow_allocs_sum_in_sati_satoshis(
        &self,
        account_key: ACCOUNT_KEY,
    ) -> Option<u128> {
        // Try to get from the delta first.
        if let Some(value) = self.delta.updated_shadow_allocs_sums.get(&account_key) {
            return Some(value.clone());
        }

        // And then try to get from the permanent in-memory states.
        self.in_memory
            .get(&account_key)
            .map(|account_body| account_body.shadow_allocs_sum)
    }

    /// Returns the account shadow allocs sum for an account key in satoshis.
    pub fn get_account_shadow_allocs_sum_in_satoshis(
        &self,
        account_key: ACCOUNT_KEY,
    ) -> Option<u64> {
        // Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_account_shadow_allocs_sum_in_sati_satoshis(account_key)?;

        // Divide by 100_000_000 to get the satoshi value.
        let satoshi_value = sati_satoshi_value / 100_000_000;

        // Return the result.
        Some(satoshi_value as u64)
    }

    /// Registers an account if it is not already registered.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_account(
        &mut self,
        account_key: ACCOUNT_KEY,
    ) -> Result<(), AccountCoinHolderRegisterError> {
        // Check if the account has just been epheremally registered in the delta.
        if self.delta.new_accounts_to_register.contains(&account_key) {
            return Err(
                AccountCoinHolderRegisterError::AccountHasJustBeenEphemerallyRegistered(
                    account_key,
                ),
            );
        }

        // Check if the account is already permanently registered.
        if self.is_account_registered(account_key) {
            return Err(
                AccountCoinHolderRegisterError::AccountIsAlreadyPermanentlyRegistered(account_key),
            );
        }

        // Insert into the new accounts to register list in the delta.
        self.delta.new_accounts_to_register.push(account_key);

        // Return the result.
        Ok(())
    }

    /// Increases the account's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_balance_up(
        &mut self,
        account_key: ACCOUNT_KEY,
        up_value_in_satoshis: u64,
    ) -> Result<(), AccountBalanceUpError> {
        // Get the old ephemeral account balance before any mutable borrows.
        let existing_account_balance_in_satoshis: u64 =
            self.get_account_balance(account_key).ok_or(
                AccountBalanceUpError::UnableToGetAccountBalance(account_key),
            )?;

        // Calculate the new ephemeral account balance.
        let new_account_balance_in_satoshis: u64 =
            existing_account_balance_in_satoshis + up_value_in_satoshis;

        // Retrieve the mutable ephemeral account balance from the delta.
        let ephemeral_account_balance =
            match self.delta.updated_account_balances.get_mut(&account_key) {
                // If the balance is already in the delta, return it.
                Some(value) => value,
                // Otherwise, from the permanent in-memory states.
                None => {
                    // Get the mutable account body from the permanent states.
                    let account_body = self.in_memory.get(&account_key).ok_or(
                        AccountBalanceUpError::UnableToGetAccountBalance(account_key),
                    )?;

                    // Insert the account balance into the delta.
                    self.delta
                        .updated_account_balances
                        .insert(account_key, account_body.balance);

                    // Get the mutable ephemeral account balance from the delta that we just inserted.
                    let ephemeral_account_balance = self
                        .delta
                        .updated_account_balances
                        .get_mut(&account_key)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the ephemeral account balance.
                    ephemeral_account_balance
                }
            };

        // Update the account balance.
        *ephemeral_account_balance = new_account_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases the account's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_balance_down(
        &mut self,
        account_key: ACCOUNT_KEY,
        down_value_in_satoshis: u64,
    ) -> Result<(), AccountBalanceDownError> {
        // Get the old ephemeral account balance before any mutable borrows.
        let existing_account_balance_in_satoshis: u64 =
            self.get_account_balance(account_key).ok_or(
                AccountBalanceDownError::UnableToGetAccountBalance(account_key),
            )?;

        // Check if the decrease would make the account balance go below zero.
        if down_value_in_satoshis > existing_account_balance_in_satoshis {
            return Err(AccountBalanceDownError::AccountBalanceWouldGoBelowZero(
                account_key,
                existing_account_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new ephemeral account balance.
        let new_account_balance_in_satoshis: u64 =
            existing_account_balance_in_satoshis - down_value_in_satoshis;

        // Retrieve the mutable ephemeral account balance from the delta.
        let ephemeral_account_balance =
            match self.delta.updated_account_balances.get_mut(&account_key) {
                // If the ephemeral account balance is already in the delta, return it.
                Some(value) => value,
                // Otherwise, from the permanent in-memory states.
                None => {
                    // Get the mutable account body from the permanent states.
                    let account_body = self.in_memory.get(&account_key).ok_or(
                        AccountBalanceDownError::UnableToGetAccountBalance(account_key),
                    )?;

                    // Insert the account balance into the delta.
                    self.delta
                        .updated_account_balances
                        .insert(account_key, account_body.balance);

                    // Get the mutable ephemeral account balance from the delta that we just inserted.
                    let ephemeral_account_balance = self
                        .delta
                        .updated_account_balances
                        .get_mut(&account_key)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the ephemeral account balance.
                    ephemeral_account_balance
                }
            };

        // Update the ephemeral account balance.
        *ephemeral_account_balance = new_account_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Increases the account's individual shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_shadow_allocs_sum_up(
        &mut self,
        account_key: ACCOUNT_KEY,
        up_value_in_sati_satoshis: u128,
    ) -> Result<(), AccountShadowAllocsSumUpError> {
        // Get the old ephemeral account shadow allocs sum before any mutable borrows.
        let existing_account_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_shadow_allocs_sum_in_sati_satoshis(account_key)
            .ok_or(AccountShadowAllocsSumUpError::UnableToGetAccountShadowAllocsSum(account_key))?;

        // Calculate the new ephemeral account shadow allocs sum.
        let new_account_shadow_allocs_sum_in_sati_satoshis: u128 =
            existing_account_shadow_allocs_sum_in_sati_satoshis + up_value_in_sati_satoshis;

        // Retrieve the mutable ephemeral account shadow allocs sum from the delta.
        let ephemeral_account_shadow_allocs_sum =
            match self.delta.updated_shadow_allocs_sums.get_mut(&account_key) {
                // If the ephemeral account shadow allocs sum is already in the delta, return it.
                Some(value) => value,
                // Otherwise, from the permanent in-memory states.
                None => {
                    // Get the mutable account body from the permanent states.
                    let account_body = self.in_memory.get(&account_key).ok_or(
                        AccountShadowAllocsSumUpError::UnableToGetAccountBody(account_key),
                    )?;

                    // Insert the account shadow allocs sum into the delta.
                    self.delta
                        .updated_shadow_allocs_sums
                        .insert(account_key, account_body.shadow_allocs_sum);

                    // Get the mutable ephemeral account shadow allocs sum from the delta that we just inserted.
                    let ephemeral_account_shadow_allocs_sum = self
                        .delta
                        .updated_shadow_allocs_sums
                        .get_mut(&account_key)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the ephemeral account shadow allocs sum.
                    ephemeral_account_shadow_allocs_sum
                }
            };

        // Update the ephemeral account shadow allocs sum.
        *ephemeral_account_shadow_allocs_sum = new_account_shadow_allocs_sum_in_sati_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases the account's individual shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_shadow_allocs_sum_down(
        &mut self,
        account_key: ACCOUNT_KEY,
        down_value_in_sati_satoshis: u128,
    ) -> Result<(), AccountShadowAllocsSumDownError> {
        // Get the old ephemeral account shadow allocs sum before any mutable borrows.
        let existing_account_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_shadow_allocs_sum_in_sati_satoshis(account_key)
            .ok_or(
                AccountShadowAllocsSumDownError::UnableToGetAccountShadowAllocsSum(account_key),
            )?;

        // Check if the decrease would make the account shadow allocs sum go below zero.
        if down_value_in_sati_satoshis > existing_account_shadow_allocs_sum_in_sati_satoshis {
            return Err(
                AccountShadowAllocsSumDownError::AccountShadowAllocsSumWouldGoBelowZero(
                    account_key,
                    existing_account_shadow_allocs_sum_in_sati_satoshis,
                    down_value_in_sati_satoshis,
                ),
            );
        }

        // Calculate the new ephemeral account shadow allocs sum.
        let new_account_shadow_allocs_sum_in_sati_satoshis: u128 =
            existing_account_shadow_allocs_sum_in_sati_satoshis - down_value_in_sati_satoshis;

        // Retrieve the mutable ephemeral account shadow allocs sum from the delta.
        let ephemeral_account_shadow_allocs_sum =
            match self.delta.updated_shadow_allocs_sums.get_mut(&account_key) {
                // If the ephemeral account shadow allocs sum is already in the delta, return it.
                Some(value) => value,
                // Otherwise, from the permanent in-memory states.
                None => {
                    // Get the mutable account body from the permanent states.
                    let account_body = self.in_memory.get(&account_key).ok_or(
                        AccountShadowAllocsSumDownError::UnableToGetAccountBody(account_key),
                    )?;

                    // Insert the account shadow allocs sum into the delta.
                    self.delta
                        .updated_shadow_allocs_sums
                        .insert(account_key, account_body.shadow_allocs_sum);

                    // Get the mutable ephemeral account shadow allocs sum from the delta that we just inserted.
                    let ephemeral_account_shadow_allocs_sum = self
                        .delta
                        .updated_shadow_allocs_sums
                        .get_mut(&account_key)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the ephemeral account shadow allocs sum.
                    ephemeral_account_shadow_allocs_sum
                }
            };

        // Update the ephemeral account shadow allocs sum.
        *ephemeral_account_shadow_allocs_sum = new_account_shadow_allocs_sum_in_sati_satoshis;

        // Return the result.
        Ok(())
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        // Restore the ephemeral states from the backup.
        self.restore_delta();
    }

    /// Clears all epheremal changes from the delta.
    pub fn flush_delta(&mut self) {
        // Clear the ephemeral states.
        self.delta.new_accounts_to_register.clear();
        self.delta.updated_account_balances.clear();

        // Clear the ephemeral states backup.
        self.delta_backup.new_accounts_to_register.clear();
        self.delta_backup.updated_account_balances.clear();
    }

    /// Applies all epheremal changes from the delta into the in-memory and on-disk.
    pub fn apply_changes(&mut self) -> Result<(), AccountCoinHolderApplyChangesError> {
        // 0. Register new accounts.
        for account_key in self.delta.new_accounts_to_register.iter() {
            // In-memory insertion.
            {
                // Construct the fresh new account body.
                let fresh_new_account_body = AccountBody {
                    balance: 0,
                    shadow_allocs_sum: 0,
                };

                // Insert the account balance into the in-memory list.
                // Register the account in-memory with zero balance.
                self.in_memory.insert(*account_key, fresh_new_account_body);
            }

            // On-disk insertion.
            {
                // Construct the fresh new concatenated value bytes.
                let fresh_new_concatenated_value_bytes: [u8; 24] = [0x00u8; 24];

                // Insert the account balance into the on-disk list.
                self.on_disk
                    .insert(account_key, fresh_new_concatenated_value_bytes.to_vec())
                    .map_err(|e| {
                        AccountCoinHolderApplyChangesError::TreeValueInsertError(
                            account_key.to_owned(),
                            0,
                            e,
                        )
                    })?;
            }
        }

        // 1. Save account balances.
        for (account_key, ephemeral_account_balance) in self.delta.updated_account_balances.iter() {
            // 1.0 In-memory insertion.
            {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory.get_mut(account_key).ok_or(
                    AccountCoinHolderApplyChangesError::UnableToGetAccountBody(*account_key),
                )?;

                // Update the balance in the in-memory states.
                account_body.balance = *ephemeral_account_balance;
            }

            // 1.1 On-disk insertion.
            {
                // Open the account tree using the account key as the tree name.
                let tree = self.on_disk.open_tree(account_key).map_err(|e| {
                    AccountCoinHolderApplyChangesError::OpenTreeError(*account_key, e)
                })?;

                // Save the balance to the balance db.
                tree.insert(
                    ACCOUNT_BALANCE_SPECIAL_KEY,
                    ephemeral_account_balance.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    AccountCoinHolderApplyChangesError::AccountBalanceValueOnDiskInsertionError(
                        account_key.to_owned(),
                        *ephemeral_account_balance,
                        e,
                    )
                })?;
            }
        }

        // 2. Save account shadow allocs sums.
        for (account_key, ephemeral_account_shadow_allocs_sum) in
            self.delta.updated_shadow_allocs_sums.iter()
        {
            // 2.0 In-memory insertion.
            {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory.get_mut(account_key).ok_or(
                    AccountCoinHolderApplyChangesError::UnableToGetAccountBody(*account_key),
                )?;

                // Update the shadow allocs sum in the in-memory states.
                account_body.shadow_allocs_sum = *ephemeral_account_shadow_allocs_sum;
            }

            // 2.1 On-disk insertion.
            {
                // Open the account tree using the account key as the tree name.
                let tree = self.on_disk.open_tree(account_key).map_err(|e| {
                    AccountCoinHolderApplyChangesError::OpenTreeError(*account_key, e)
                })?;

                tree.insert(
                    ACCOUNT_ALLOCS_SUM_SPECIAL_KEY,
                    ephemeral_account_shadow_allocs_sum.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    AccountCoinHolderApplyChangesError::AccountShadowAllocsSumValueOnDiskInsertionError(
                        account_key.to_owned(),
                        *ephemeral_account_shadow_allocs_sum,
                        e,
                    )
                })?;
            }
        }

        // Clear the delta.
        self.flush_delta();

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
