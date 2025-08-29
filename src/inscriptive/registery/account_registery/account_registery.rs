use super::account_registery_error::{
    AccountRegisteryConstructionError, AccountRegisteryRegisterError, AccountRegisterySaveAllError,
};
use crate::{
    constructive::{
        entity::account::account::Account, valtype::val::short_val::short_val::ShortVal,
    },
    inscriptive::registery::account_registery::account_registery_error::AccountRegisteryIncrementCallCounterError,
    operative::Chain,
};
use secp::Point;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded registery of accounts.
#[allow(non_camel_case_types)]
pub type ACCOUNT_REGISTERY = Arc<Mutex<AccountRegistery>>;

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Registery index of an account for efficient referencing (from 1 to U32::MAX).
#[allow(non_camel_case_types)]
type REGISTERY_INDEX = u32;

/// Call counter of an account used to rank accounts.
#[allow(non_camel_case_types)]
type CALL_COUNTER = u64;

/// Rank integer representing the rank position of an account (from 1 to U32::MAX).
#[allow(non_camel_case_types)]
type RANK = u32;

/// In-block local call counter of an account used to increment the call counter.
#[allow(non_camel_case_types)]
type IN_BLOCK_CALL_COUNTER = u16;

/// Body of an account.
struct AccountBody {
    pub registery_index: u32,
    pub call_counter: u64,
}

impl AccountBody {
    /// Updates the call counter of an account.
    pub fn update_call_counter(&mut self, new_call_counter: u64) {
        self.call_counter = new_call_counter;
    }
}

/// Directory for storing accounts and their call counters.
/// There are two in-memory lists, one by registery index and one by rank.
pub struct AccountRegistery {
    // In-memory cache of accounts by registery index.
    in_memory_accounts: HashMap<ACCOUNT_KEY, AccountBody>,
    // In-memory cache of accounts by rank.
    in_memory_ranks: HashMap<RANK, ACCOUNT_KEY>,

    // On-disk db for storing the accounts and their call counters.
    on_disk_db: sled::Db,

    // Ephemeral states
    epheremal_accounts_to_register: Vec<ACCOUNT_KEY>,
    epheremal_accounts_to_increment: HashMap<ACCOUNT_KEY, IN_BLOCK_CALL_COUNTER>,

    // Backups
    backup_of_ephemeral_accounts_to_register: Vec<ACCOUNT_KEY>,
    backup_of_ephemeral_accounts_to_increment: HashMap<ACCOUNT_KEY, IN_BLOCK_CALL_COUNTER>,
}

impl AccountRegistery {
    pub fn new(chain: Chain) -> Result<ACCOUNT_REGISTERY, AccountRegisteryConstructionError> {
        // Construct the accounts db path.
        let account_registery_path =
            format!("{}/{}/{}", "db", chain.to_string(), "registery/account");

        // Open the accounts db.
        let account_registery_db = {
            sled::open(account_registery_path)
                .map_err(AccountRegisteryConstructionError::AccountsDBOpenError)?
        };

        // Initialize the in-memory list of accounts.
        let mut accounts = HashMap::<ACCOUNT_KEY, AccountBody>::new();

        // Iterate over all items in the db.
        for tree_name in account_registery_db.tree_names() {
            // Convert the tree name to a contract id.
            let account_key: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                AccountRegisteryConstructionError::InvalidAccountKeyBytes(tree_name.to_vec())
            })?;

            // Initialize the registery index and call counter.
            let mut registery_index = 0;

            // Initialize the call counter.
            let mut call_counter = 0;

            // Open the account registery tree.
            let tree = account_registery_db.open_tree(&tree_name).map_err(|e| {
                AccountRegisteryConstructionError::AccountRegisteryTreeOpenError(account_key, e)
            })?;

            // Iterate over all items in the account registery tree.
            for item in tree.iter() {
                // Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(
                            AccountRegisteryConstructionError::AccountRegisteryTreeOpenError(
                                account_key,
                                e,
                            ),
                        );
                    }
                };

                // Convert the key to a key byte.
                let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    AccountRegisteryConstructionError::InvalidAccountKeyBytes(key.to_vec())
                })?;

                // Match the key byte.
                match key_byte[0] {
                    // 0x00 key byte represents the registery index.
                    0u8 => {
                        // Convert the value to a registery index bytes.
                        let registery_index_bytes: [u8; 4] =
                            value.as_ref().try_into().map_err(|_| {
                                AccountRegisteryConstructionError::InvalidRegisteryIndexBytes(
                                    value.to_vec(),
                                )
                            })?;

                        // Convert the registery index bytes to a registery index.
                        registery_index = u32::from_le_bytes(registery_index_bytes);
                    }

                    // 0x01 key byte represents the call counter.
                    1u8 => {
                        // Convert the value to a call counter bytes.
                        let call_counter_bytes: [u8; 8] =
                            value.as_ref().try_into().map_err(|_| {
                                AccountRegisteryConstructionError::InvalidCallCounterBytes(
                                    value.to_vec(),
                                )
                            })?;

                        // Convert the call counter bytes to a call counter.
                        call_counter = u64::from_le_bytes(call_counter_bytes);
                    }

                    _ => {
                        return Err(AccountRegisteryConstructionError::InvalidKeyByte(
                            key.to_vec(),
                        ));
                    }
                }

                // Construct the account body.
                let account_body = AccountBody {
                    registery_index,
                    call_counter,
                };

                // Insert the account body into the in-memory list of accounts.
                accounts.insert(account_key, account_body);
            }
        }

        // Rank the accounts by call counter (descending) and registry index (ascending as tiebreaker).
        let ranked = Self::rank_accounts(&accounts);

        // Construct the account registery.
        let account_registery = AccountRegistery {
            in_memory_accounts: accounts,
            in_memory_ranks: ranked,
            on_disk_db: account_registery_db,
            epheremal_accounts_to_register: Vec::new(),
            epheremal_accounts_to_increment: HashMap::new(),
            backup_of_ephemeral_accounts_to_register: Vec::new(),
            backup_of_ephemeral_accounts_to_increment: HashMap::new(),
        };

        // Guard the account registery.
        let guarded_account_registery = Arc::new(Mutex::new(account_registery));

        // Return the guarded account registery.
        Ok(guarded_account_registery)
    }

    /// Ranks accounts by call counter (descending) and registry index (ascending as tiebreaker).
    /// Returns a HashMap where keys are ranks starting from 1.
    fn rank_accounts(accounts: &HashMap<ACCOUNT_KEY, AccountBody>) -> HashMap<RANK, ACCOUNT_KEY> {
        // Convert to vector for sorting
        let mut account_vec: Vec<(&ACCOUNT_KEY, &AccountBody)> = accounts.iter().collect();

        // Sort by call counter (descending), then by registry index (ascending) as tiebreaker
        account_vec.sort_by(|a, b| {
            // Primary sort: call counter (descending)
            b.1.call_counter
                .cmp(&a.1.call_counter)
                // Secondary sort: registry index (ascending) as tiebreaker
                .then(a.1.registery_index.cmp(&b.1.registery_index))
        });

        // Convert to ranked HashMap with ranks starting from 1
        let mut ranked_accounts = HashMap::<RANK, ACCOUNT_KEY>::new();
        for (index, (account_key, _)) in account_vec.into_iter().enumerate() {
            let rank = (index + 1) as RANK; // Start from 1
            ranked_accounts.insert(rank, *account_key);
        }

        ranked_accounts
    }

    /// Checks if an account is registered.
    pub fn is_account_registered(&self, account_key: ACCOUNT_KEY) -> bool {
        // Try from ephemeral states first.
        if self.epheremal_accounts_to_register.contains(&account_key) {
            return true;
        }

        // Try from in-memory states next.
        self.in_memory_accounts.contains_key(&account_key)
    }

    /// Returns the rank of an account by its key.
    pub fn get_rank_by_account_key(&self, account_key: ACCOUNT_KEY) -> Option<RANK> {
        // Iterate ranked list and return the rank of the account key.
        for (rank, key) in self.in_memory_ranks.iter() {
            // If the key matches, return the rank.
            if key == &account_key {
                return Some(*rank);
            }
        }

        // If the account is not found, return None.
        None
    }

    /// Returns the account key by its rank.
    pub fn get_account_key_by_rank(&self, rank: RANK) -> Option<ACCOUNT_KEY> {
        // Return the account key by the rank.
        self.in_memory_ranks.get(&rank).cloned()
    }

    /// Returns the account by its key.
    pub fn get_account_info_by_account_key(
        &self,
        account_key: ACCOUNT_KEY,
    ) -> Option<(REGISTERY_INDEX, CALL_COUNTER, RANK)> {
        // Return the account body by the account key.
        let account_body = self.in_memory_accounts.get(&account_key)?;
        let rank = self.get_rank_by_account_key(account_key)?;
        let registery_index = account_body.registery_index;
        let call_counter = account_body.call_counter;
        Some((registery_index, call_counter, rank))
    }

    /// Returns the account by its key.
    pub fn get_account_by_account_key(&self, account_key: ACCOUNT_KEY) -> Option<Account> {
        let account_body = self.in_memory_accounts.get(&account_key)?;
        let rank = self.get_rank_by_account_key(account_key)?;
        let registery_index = account_body.registery_index;
        Some(Account {
            key: Point::from_slice(&account_key).unwrap(),
            registery_index: Some(ShortVal::new(registery_index as u32)),
            rank: Some(ShortVal::new(rank as u32)),
        })
    }

    /// Returns the account by its rank.
    pub fn get_account_by_rank(&self, rank: RANK) -> Option<Account> {
        // Return the account key by the rank.
        let account_key = self.in_memory_ranks.get(&rank).cloned()?;
        let account_body = self.in_memory_accounts.get(&account_key)?;
        let registery_index = account_body.registery_index;

        Some(Account {
            key: Point::from_slice(&account_key).unwrap(),
            registery_index: Some(ShortVal::new(registery_index as u32)),
            rank: Some(ShortVal::new(rank as u32)),
        })
    }

    /// Clones ephemeral states into the backup.
    fn backup_ephemeral_states(&mut self) {
        self.backup_of_ephemeral_accounts_to_register = self.epheremal_accounts_to_register.clone();
        self.backup_of_ephemeral_accounts_to_increment =
            self.epheremal_accounts_to_increment.clone();
    }

    /// Prepares the registery for the next execution.
    pub fn pre_execution(&mut self) {
        // Backup the ephemeral states.
        self.backup_ephemeral_states();
    }

    /// Epheremally registers an account to the registery.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn register_account(
        &mut self,
        account_key: ACCOUNT_KEY,
    ) -> Result<(), AccountRegisteryRegisterError> {
        // If the account is already registered, return an error.
        if self.is_account_registered(account_key) {
            return Err(
                AccountRegisteryRegisterError::AccountAlreadyPermanentlyRegistered(account_key),
            );
        }

        // If the account is already pushed to epheremal list, return an error.
        if self.epheremal_accounts_to_register.contains(&account_key) {
            return Err(
                AccountRegisteryRegisterError::AccountAlreadyEphemerallyRegistered(account_key),
            );
        }

        // Push the account to the ephemeral list.
        self.epheremal_accounts_to_register.push(account_key);

        // Return the result.
        Ok(())
    }

    /// Epheremally increments the call counter of an account.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn increment_account_call_counter(
        &mut self,
        account_key: ACCOUNT_KEY,
        optimized: bool,
    ) -> Result<(), AccountRegisteryIncrementCallCounterError> {
        // If not optimized, check if the account is registered.
        if !optimized {
            if !self.is_account_registered(account_key) {
                return Err(
                    AccountRegisteryIncrementCallCounterError::AccountNotRegistered(account_key),
                );
            }
        }

        // Try to get the in-block call counter from the epheremal list.
        let in_block_call_counter = match self.epheremal_accounts_to_increment.get(&account_key) {
            Some(value) => *value,
            None => 0,
        };

        // Increment the call counter.
        let new_in_block_call_counter = in_block_call_counter + 1;

        // Insert the new call counter into the epheremal list.
        self.epheremal_accounts_to_increment
            .insert(account_key, new_in_block_call_counter);

        // Return the result.
        Ok(())
    }

    /// Restores ephemeral states from the backup.
    fn restore_ephemeral_states(&mut self) {
        self.epheremal_accounts_to_register = self.backup_of_ephemeral_accounts_to_register.clone();
        self.epheremal_accounts_to_increment =
            self.backup_of_ephemeral_accounts_to_increment.clone();
    }

    /// Restores the last ephemeral state.
    pub fn rollback_last(&mut self) {
        self.restore_ephemeral_states();
    }

    /// Clears all ephemeral states.
    pub fn rollback_all(&mut self) {
        // Clear the ephemeral states.
        self.epheremal_accounts_to_register.clear();
        self.epheremal_accounts_to_increment.clear();

        // Clear the backup.
        self.backup_of_ephemeral_accounts_to_register.clear();
        self.backup_of_ephemeral_accounts_to_increment.clear();
    }

    /// Returns the height of the registery index.
    fn registery_index_height(&self) -> u32 {
        self.in_memory_accounts.len() as u32
    }

    /// Saves all ephemeral states to in-memory and on-disk.
    pub fn save_all(&mut self) -> Result<(), AccountRegisterySaveAllError> {
        // Get the height of the registery index.
        let registery_index_height = self.registery_index_height();

        // Register the accounts.
        for (index, account_key) in self.epheremal_accounts_to_register.iter().enumerate() {
            // Calculate the registery index.
            let registery_index = registery_index_height + index as u32;

            // Initial call counter value is set to zero.
            let initial_call_counter = 0;

            // Save-in-memory:
            {
                // Construct the account body.
                let account_body = AccountBody {
                    registery_index,
                    call_counter: initial_call_counter,
                };

                // Insert the account body into the in-memory list.
                self.in_memory_accounts.insert(*account_key, account_body);
            }

            // Save-on-disk:
            {
                // Open the tree for the account.
                let on_disk_account_tree = self.on_disk_db.open_tree(account_key).map_err(|e| {
                    AccountRegisterySaveAllError::UnableToOpenAccountTree(*account_key, e)
                })?;

                // Get the registery index bytes.
                let registery_index_bytes = registery_index.to_le_bytes().to_vec();

                // Insert the registery index into the tree.
                // 0x00 key byte represents the registery index.
                on_disk_account_tree
                    .insert([0x00u8; 1], registery_index_bytes)
                    .map_err(|e| {
                        AccountRegisterySaveAllError::UnableToInsertRegisteryIndex(*account_key, e)
                    })?;

                // Fresh new call counter bytes.
                let initial_call_counter_bytes = initial_call_counter.to_le_bytes().to_vec();

                // Insert the call counter into the tree.
                // 0x01 key byte represents the call counter.
                on_disk_account_tree
                    .insert([0x01u8; 1], initial_call_counter_bytes)
                    .map_err(|e| {
                        AccountRegisterySaveAllError::UnableToInsertCallCounter(*account_key, e)
                    })?;
            }
        }

        // Increment the call counter of the accounts.
        for (account_key, in_block_call_counter) in self.epheremal_accounts_to_increment.iter() {
            // Get the mutable account body from the in-memory list.
            let in_memory_account_body = self.in_memory_accounts.get_mut(account_key).ok_or(
                AccountRegisterySaveAllError::UnableToGetAccountCallCounter(*account_key),
            )?;

            // Get the historical call counter.
            let historical_call_counter = in_memory_account_body.call_counter;

            // Calculate the new call counter.
            let new_call_counter = historical_call_counter + *in_block_call_counter as u64;

            // Save in-memory:
            {
                // Update the call counter.
                in_memory_account_body.update_call_counter(new_call_counter);
            }

            // Save on-disk:
            {
                // Open the tree for the account.
                let on_disk_account_tree = self.on_disk_db.open_tree(account_key).map_err(|e| {
                    AccountRegisterySaveAllError::UnableToOpenAccountTree(*account_key, e)
                })?;

                // Get the call counter bytes.
                let new_call_counter_bytes = new_call_counter.to_le_bytes().to_vec();

                // Insert the new call counter into the tree.
                // 0x01 key byte represents the call counter.
                on_disk_account_tree
                    .insert([0x01u8; 1], new_call_counter_bytes)
                    .map_err(|e| {
                        AccountRegisterySaveAllError::UnableToInsertCallCounter(*account_key, e)
                    })?;
            }
        }

        // Rank the accounts by call counter (descending) and registry index (ascending as tiebreaker).
        let new_ranked_accounts = Self::rank_accounts(&self.in_memory_accounts);

        // Update the ranked list.
        self.in_memory_ranks = new_ranked_accounts;

        // Return the result.
        Ok(())
    }
}
