use super::contract_coin_holder_error::{
    ContractCoinHolderConstructionError, ContractCoinHolderSaveError,
};
use crate::inscriptive::coin_holder::contract_coin_holder::contract_coin_holder_error::{
    ContractBalanceDownError, ContractBalanceUpError, ContractCoinHolderRegisterError,
    ShadowAllocDownAllError, ShadowAllocDownError, ShadowAllocError, ShadowAllocUpAllError,
    ShadowAllocUpError, ShadowDeallocError,
};
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// A custom, high-precision satoshi amount.
/// 1 satoshi = 100,000,000 sati-satoshis.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// A struct for representing a shadow space of a contract.
#[derive(Clone)]
struct ContractShadowSpace {
    // Total allocated BTC value of the entire shadow space.
    pub allocs_sum: SATOSHI_AMOUNT,
    // Allocated BTC values of each account.
    pub allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
}

/// A struct for containing BTC balance and shadow space allocations of a contract.
#[derive(Clone)]
struct ContractBody {
    // Contract's BTC balance.
    pub balance: u64,
    // Contract's shadow space.
    pub shadow_space: ContractShadowSpace,
}

/// A database manager for handling contract balances and shadow spaces.
/// For now, we are caching everything in memory.
pub struct ContractCoinHolder {
    // IN-MEMORY STATES
    /// In-memory cache of states: CONTRACT_ID -> ContractBody
    in_memory: HashMap<CONTRACT_ID, ContractBody>,

    // ON-DISK STATES
    /// Sled DB with all contract balances in a single tree.
    balance_db: sled::Db,
    /// Sled DB with all contract shadow spaces in a single tree.
    shadow_space_db: sled::Db,

    // EPHEMERAL STATES
    /// In-memory cache of ephemeral contract balances.
    ephemeral_balances: HashMap<CONTRACT_ID, u64>,
    /// In-memory cache of ephemeral contract shadow spaces.
    ephemeral_shadow_spaces: HashMap<CONTRACT_ID, ContractShadowSpace>,
    /// In-memory cache of ephemeral deallocs to remove.
    epheremal_dealloc_list: HashMap<CONTRACT_ID, Vec<ACCOUNT_KEY>>,

    // BACKUPS
    /// In-memory cache of ephemeral contract balances backup.
    backup_of_ephemeral_balances: HashMap<CONTRACT_ID, u64>,
    /// In-memory cache of ephemeral contract shadow spaces backup.
    backup_of_ephemeral_shadow_spaces: HashMap<CONTRACT_ID, ContractShadowSpace>,
    /// In-memory cache of ephemeral deallocs to remove backup.
    backup_of_ephemeral_dealloc_list: HashMap<CONTRACT_ID, Vec<ACCOUNT_KEY>>,
}

/// Guarded contract coin holder.
#[allow(non_camel_case_types)]
pub type CONTRACT_COIN_HOLDER = Arc<Mutex<ContractCoinHolder>>;

// TODO: Implement a rank-based caching mechanism to only cache the high-ranked states.
// Right now, we are caching *ALL* contract states in memory.
impl ContractCoinHolder {
    /// Initialize the state for the given chain
    pub fn new(chain: Chain) -> Result<CONTRACT_COIN_HOLDER, ContractCoinHolderConstructionError> {
        // Open the balance db.
        let balance_path = format!("db/{}/coin/contract/balance", chain.to_string());
        let balance_db = sled::open(balance_path)
            .map_err(ContractCoinHolderConstructionError::BalancesDBOpenError)?;

        // Open the shadow space db.
        let shadow_space_path = format!("db/{}/coin/contract/shadow_space", chain.to_string());
        let shadow_space_db = sled::open(shadow_space_path)
            .map_err(ContractCoinHolderConstructionError::ShadowSpaceDBOpenError)?;

        // Initialize the in-memory cache of contract bodies.
        let mut in_memory = HashMap::<CONTRACT_ID, ContractBody>::new();

        // Iterate over all contract shadow spaces in the shadow space db.
        for tree_name in shadow_space_db.tree_names() {
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                ContractCoinHolderConstructionError::InvalidContractIDBytes(tree_name.to_vec())
            })?;

            // Open the contract shadow space tree.
            let shadow_space_tree = shadow_space_db.open_tree(&tree_name).map_err(|e| {
                ContractCoinHolderConstructionError::ShadowSpaceTreeOpenError(contract_id, e)
            })?;

            // Initialize the in-memory cache of contract shadow space.
            let mut allocs = HashMap::<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>::new();

            // On-disk stored allocs sum.
            let mut allocs_sum: u64 = 0;

            // Iterate over all items in the contract tree.
            for alloc in shadow_space_tree.iter() {
                // Get the key and value.
                let (alloc_account_key, alloc_value) = match alloc {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(
                            ContractCoinHolderConstructionError::ContractShadowIterError(e),
                        );
                    }
                };

                // Convert the key to an allocation account key.
                let alloc_account_key: [u8; 32] =
                    alloc_account_key.to_vec().try_into().map_err(|_| {
                        ContractCoinHolderConstructionError::InvalidShadowAccountKey(
                            alloc_account_key.to_vec(),
                        )
                    })?;

                // Match the account key.
                match alloc_account_key == [0xff; 32] {
                    false => {
                        // This key is a normal account key that corresponds to an account allocation.

                        // Convert the value to an allocation value.
                        let alloc_value_in_sati_satoshis: u128 =
                            u128::from_le_bytes(alloc_value.as_ref().try_into().map_err(|_| {
                                ContractCoinHolderConstructionError::InvalidShadowAllocValueBytes(
                                    alloc_value.to_vec(),
                                )
                            })?);

                        // Update the shadow space allocations.
                        allocs.insert(alloc_account_key, alloc_value_in_sati_satoshis);
                    }
                    true => {
                        // This key (0xff..) is a special key that corresponds to the allocs sum value.

                        // Convert the value to an allocation value.
                        let allocs_sum_value_in_satoshis: u64 =
                            u64::from_le_bytes(alloc_value.as_ref().try_into().map_err(|_| {
                                ContractCoinHolderConstructionError::InvalidShadowAllocsSumBytes(
                                    alloc_value.to_vec(),
                                )
                            })?);

                        // Update the shadow space allocations sum.
                        allocs_sum = allocs_sum_value_in_satoshis;
                    }
                }
            }

            // Get the contract balance from the balance db.
            let contract_balance = {
                // Get the balance value using contract_id as the key
                match balance_db.get(&contract_id) {
                    Ok(Some(balance_bytes)) => {
                        u64::from_le_bytes(balance_bytes.as_ref().try_into().map_err(|_| {
                            ContractCoinHolderConstructionError::InvalidBalanceBytesError(
                                balance_bytes.to_vec(),
                            )
                        })?)
                    }
                    Ok(None) => {
                        return Err(
                            ContractCoinHolderConstructionError::UnableToGetContractBalance(
                                contract_id,
                                None,
                            ),
                        );
                    }
                    Err(e) => {
                        return Err(
                            ContractCoinHolderConstructionError::UnableToGetContractBalance(
                                contract_id,
                                Some(e),
                            ),
                        );
                    }
                }
            };

            // Check if the shadow space allocations sum exceeds the contract balance.
            if allocs_sum > contract_balance {
                return Err(
                    ContractCoinHolderConstructionError::AllocsSumExceedsTheContractBalance(
                        contract_id,
                        allocs_sum,
                        contract_balance,
                    ),
                );
            }

            // Create the contract body.
            let contract_body = ContractBody {
                balance: contract_balance,
                shadow_space: ContractShadowSpace { allocs_sum, allocs },
            };

            // Insert the contract body into the in-memory cache.
            in_memory.insert(contract_id, contract_body);
        }

        // Create the contract coin holder.
        let contract_coin_holder = ContractCoinHolder {
            in_memory,
            balance_db,
            shadow_space_db,
            ephemeral_balances: HashMap::<CONTRACT_ID, u64>::new(),
            ephemeral_shadow_spaces: HashMap::<CONTRACT_ID, ContractShadowSpace>::new(),
            epheremal_dealloc_list: HashMap::<CONTRACT_ID, Vec<ACCOUNT_KEY>>::new(),
            backup_of_ephemeral_balances: HashMap::<CONTRACT_ID, u64>::new(),
            backup_of_ephemeral_shadow_spaces: HashMap::<CONTRACT_ID, ContractShadowSpace>::new(),
            backup_of_ephemeral_dealloc_list: HashMap::<CONTRACT_ID, Vec<ACCOUNT_KEY>>::new(),
        };

        // Create the guarded contract coin holder.
        let guarded_contract_coin_holder = Arc::new(Mutex::new(contract_coin_holder));

        // Return the guarded contract coin holder.
        Ok(guarded_contract_coin_holder)
    }

    /// Clones ephemeral states into the backup.
    fn backup_ephemeral_states(&mut self) {
        self.backup_of_ephemeral_balances = self.ephemeral_balances.clone();
        self.backup_of_ephemeral_shadow_spaces = self.ephemeral_shadow_spaces.clone();
        self.backup_of_ephemeral_dealloc_list = self.epheremal_dealloc_list.clone();
    }

    /// Restores ephemeral states from the backup.
    fn restore_ephemeral_states(&mut self) {
        self.ephemeral_balances = self.backup_of_ephemeral_balances.clone();
        self.ephemeral_shadow_spaces = self.backup_of_ephemeral_shadow_spaces.clone();
        self.epheremal_dealloc_list = self.backup_of_ephemeral_dealloc_list.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the ephemeral states.
        self.backup_ephemeral_states();
    }

    /// Checks if a contract is registered.
    pub fn in_contract_registered(&self, contract_id: [u8; 32]) -> bool {
        self.in_memory.contains_key(&contract_id)
    }

    /// Get the contract coin balance for a contract ID.
    pub fn get_contract_balance(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some(balance) = self.ephemeral_balances.get(&contract_id) {
            return Some(*balance);
        }

        // And then try to read from the in-memory states.
        self.in_memory.get(&contract_id).map(|body| body.balance)
    }

    /// Get the contract shadow allocation for a contract ID.
    pub fn get_contract_allocs_sum(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to read from the ephemeral states first.
        if let Some(shadow_space) = self.ephemeral_shadow_spaces.get(&contract_id) {
            return Some(shadow_space.allocs_sum);
        }

        // And then try to get from the in-memory states.
        self.in_memory
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs_sum)
    }

    /// Get the number of total shadow allocations of the contract.
    pub fn get_contract_num_allocs(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some(shadow_space) = self.ephemeral_shadow_spaces.get(&contract_id) {
            return Some(shadow_space.allocs.len() as u64);
        }

        // And then try to get from the in-memory states.
        self.in_memory
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs.len() as u64)
    }

    /// Get the shadow allocation value of an account for a specific contract ID.
    pub fn get_account_shadow_alloc_value_in_sati_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Option<u128> {
        // If this account is JUST deallocated, return none.
        if let Some(dealloc_list) = self.epheremal_dealloc_list.get(&contract_id) {
            if dealloc_list.contains(&account_key) {
                return None;
            }
        }

        // Try to get from the ephemeral states first.
        if let Some(shadow_space) = self.ephemeral_shadow_spaces.get(&contract_id) {
            return shadow_space.allocs.get(&account_key).cloned();
        }

        // And then try to get from the in-memory states.
        self.in_memory
            .get(&contract_id)
            .and_then(|body| body.shadow_space.allocs.get(&account_key).cloned())
    }

    /// Get the shadow allocation value of an account for a specific contract ID in satoshis.
    pub fn get_account_shadow_alloc_value_in_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Option<u64> {
        // Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)?;

        // Divide by 100_000_000 to get the satoshi value.
        let satoshi_value = sati_satoshi_value / 100_000_000;

        // Return the result.
        Some(satoshi_value as u64)
    }

    /// Registers a contract if it is not already registered.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn register_contract(
        &mut self,
        contract_id: [u8; 32],
    ) -> Result<(), ContractCoinHolderRegisterError> {
        // Check if the contract ID is already registered.
        if self.in_memory.contains_key(&contract_id) {
            return Err(ContractCoinHolderRegisterError::ContractAlreadyRegistered(
                contract_id,
            ));
        }

        // Insert into the epheremal balances with zero balance.
        self.ephemeral_balances.insert(contract_id, 0);

        // Create a fresh new shadow space with zero allocs sum.
        let fresh_new_shadow_space = ContractShadowSpace {
            allocs_sum: 0,
            allocs: HashMap::new(),
        };

        // Insert into the epheremal shadow spaces.
        self.ephemeral_shadow_spaces
            .insert(contract_id, fresh_new_shadow_space);

        // Return the result.
        Ok(())
    }

    /// Increases a contract balance by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn contract_balance_up(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<(), ContractBalanceUpError> {
        // Get the old contract balance before any mutable borrows.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                ContractBalanceUpError::UnableToGetContractBalance(contract_id),
            )?;

        // Calculate the new contract balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis + up_value_in_satoshis;

        // Retrieve the mutable balance from the ephemeral states.
        let ephemeral_contract_balance = match self.ephemeral_balances.get_mut(&contract_id) {
            // If the balance is already in the ephemeral states, return it.
            Some(balance) => balance,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable balance from the permanent in-memory states.
                let contract_body = self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ContractBalanceUpError::ContractBodyNotFound(contract_id))?;

                // Get the mutable balance.
                let balance = contract_body.balance;

                // Insert the balance into the ephemeral states.
                self.ephemeral_balances.insert(contract_id, balance);

                // Get the mutable balance from the ephemeral that we just inserted.
                let ephemeral_balance = self
                    .ephemeral_balances
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the balance.
                ephemeral_balance
            }
        };

        // Update the contract balance.
        *ephemeral_contract_balance = new_contract_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases a contract balance by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn contract_balance_down(
        &mut self,
        contract_id: [u8; 32],
        down_value_in_satoshis: u64,
    ) -> Result<(), ContractBalanceDownError> {
        // Get the old contract balance before any mutable borrows.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                ContractBalanceDownError::UnableToGetContractBalance(contract_id),
            )?;

        // Check if the decrease would make the contract balance go below zero.
        if down_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(ContractBalanceDownError::ContractBalanceWouldGoBelowZero(
                contract_id,
                existing_contract_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new contract balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis - down_value_in_satoshis;

        // Retrieve the mutable balance from the ephemeral states.
        let ephemeral_contract_balance = match self.ephemeral_balances.get_mut(&contract_id) {
            // If the balance is already in the ephemeral states, return it.
            Some(balance) => balance,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable balance from the permanent in-memory states.
                let contract_body = self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ContractBalanceDownError::ContractBodyNotFound(contract_id))?;

                // Get the mutable balance.
                let balance = contract_body.balance;

                // Insert the balance into the ephemeral states.
                self.ephemeral_balances.insert(contract_id, balance);

                // Get the mutable balance from the ephemeral that we just inserted.
                let ephemeral_balance = self
                    .ephemeral_balances
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the balance.
                ephemeral_balance
            }
        };

        // Update the contract balance.
        *ephemeral_contract_balance = new_contract_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    // Creates an allocation space for an account in a contract shadow space.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn shadow_alloc(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Result<(), ShadowAllocError> {
        // Check if the account key is already allocated.
        if self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .is_some()
        {
            return Err(ShadowAllocError::AccountKeyAlreadyAllocated(
                contract_id,
                account_key,
            ));
        }

        // Try to get the shadow space from ephemeral states first.
        let ephemeral_contract_shadow_space =
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                Some(shadow_space) => shadow_space,
                None => {
                    // Otherwise, from the permanent in-memory states.
                    let contract_body = self
                        .in_memory
                        .get_mut(&contract_id)
                        .ok_or(ShadowAllocError::ShadowSpaceNotFound(contract_id))?;

                    // Get the mutable shadow space.
                    let shadow_space = contract_body.shadow_space.clone();

                    // Insert the shadow space into the ephemeral states.
                    self.ephemeral_shadow_spaces
                        .insert(contract_id, shadow_space);

                    // Get the mutable shadow space from the epheremal that we just inserted.
                    let ephemeral_shadow_space = self
                        .ephemeral_shadow_spaces
                        .get_mut(&contract_id)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the shadow space.
                    ephemeral_shadow_space
                }
            };

        // Insert the account key into the ephemeral shadow space.
        ephemeral_contract_shadow_space
            .allocs
            .insert(account_key, 0);

        // Return the result.
        Ok(())
    }

    /// Deallocates an account in a contract shadow space.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn shadow_dealloc(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Result<(), ShadowDeallocError> {
        // Get the account allocation value.
        let allocation_value_in_sati_satoshis = self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(ShadowDeallocError::UnableToGetAccountAllocValue(
                contract_id,
                account_key,
            ))?;

        // Check if the account allocation value is non-zero.
        // Deallocation is alloawed only if the allocation value is zero.
        if allocation_value_in_sati_satoshis != 0 {
            return Err(ShadowDeallocError::AlocValueIsNonZero(
                contract_id,
                account_key,
            ));
        }

        // Get the mutable dealloc list.
        let dealloc_list = self
            .epheremal_dealloc_list
            .get_mut(&contract_id)
            .ok_or(ShadowDeallocError::UnableToGetDeallocList(contract_id))?;

        // Cheack if this account already ephmeral deallocated.
        if dealloc_list.contains(&account_key) {
            return Err(ShadowDeallocError::AccountKeyAlreadyEphemerallyDeallocated(
                contract_id,
                account_key,
            ));
        }

        // Insert the account key into the ephemeral dealloc list.
        dealloc_list.push(account_key);

        // Return the result.
        Ok(())
    }

    /// Inserts or updates a shadow allocation value by key and contract ID ephemerally.    
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn shadow_alloc_up(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
        up_value_in_satoshis: u64,
    ) -> Result<(), ShadowAllocUpError> {
        // Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 = (up_value_in_satoshis as u128) * 100_000_000;

        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(ShadowAllocUpError::UnableToGetOldAccountAllocValue(
                contract_id,
                account_key,
            ))?;

        let existing_contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(ShadowAllocUpError::UnableToGetContractBalance(contract_id))?;

        // Retrieve the mutable shadow space from the ephemeral states.
        let ephemeral_contract_shadow_space =
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                // If the shadow space is already in the ephemeral states, return it.
                Some(shadow_space) => shadow_space,
                // Otherwise, from the permanent in-memory states.
                None => {
                    let contract_body = self
                        .in_memory
                        .get_mut(&contract_id)
                        .ok_or(ShadowAllocUpError::ShadowSpaceNotFound(contract_id))?;

                    // Get the mutable shadow space.
                    let shadow_space = contract_body.shadow_space.clone();

                    // Insert the shadow space into the ephemeral states.
                    self.ephemeral_shadow_spaces
                        .insert(contract_id, shadow_space);

                    // Get the mutable shadow space from the epheremal that we just inserted.
                    let ephemeral_shadow_space = self
                        .ephemeral_shadow_spaces
                        .get_mut(&contract_id)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the shadow space.
                    ephemeral_shadow_space
                }
            };

        // Calculate the new allocation value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            existing_account_shadow_alloc_value_in_sati_satoshis + up_value_in_sati_satoshis;

        // Calculate the new allocation sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            ephemeral_contract_shadow_space.allocs_sum + up_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(ShadowAllocUpError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                existing_contract_balance_in_satoshis,
            ));
        }

        // Insert (or update) the account shadow allocation value into the ephemeral states.
        ephemeral_contract_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // Update the contract shadow allocation sum value.
        ephemeral_contract_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases a shadow allocation value by key and contract ID ephemerally.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn shadow_alloc_down(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
        down_value_in_satoshis: u64,
    ) -> Result<(), ShadowAllocDownError> {
        // Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 = (down_value_in_satoshis as u128) * 100_000_000;

        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(ShadowAllocDownError::UnableToGetOldAccountAllocValue(
                contract_id,
                account_key,
            ))?;

        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                ShadowAllocDownError::UnableToGetContractBalance(contract_id),
            )?;

        // Check if the decrease would make the allocation value go below zero.
        if down_value_in_sati_satoshis > existing_account_shadow_alloc_value_in_sati_satoshis {
            return Err(ShadowAllocDownError::AllocValueWouldGoBelowZero(
                contract_id,
                account_key,
                existing_account_shadow_alloc_value_in_sati_satoshis,
                down_value_in_sati_satoshis,
            ));
        }

        // Retrieve the mutable shadow space from the ephemeral states.
        let ephemeral_contract_shadow_space =
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                // If the shadow space is already in the ephemeral states, return it.
                Some(shadow_space) => shadow_space,
                // Otherwise, from the permanent in-memory states.
                None => {
                    let contract_body = self
                        .in_memory
                        .get_mut(&contract_id)
                        .ok_or(ShadowAllocDownError::ShadowSpaceNotFound(contract_id))?;

                    // Get the mutable shadow space.
                    let shadow_space = contract_body.shadow_space.clone();

                    // Insert the shadow space into the ephemeral states.
                    self.ephemeral_shadow_spaces
                        .insert(contract_id, shadow_space);

                    // Get the mutable shadow space from the epheremal that we just inserted.
                    let ephemeral_shadow_space = self
                        .ephemeral_shadow_spaces
                        .get_mut(&contract_id)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the shadow space.
                    ephemeral_shadow_space
                }
            };

        // Calculate the new allocation value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            existing_account_shadow_alloc_value_in_sati_satoshis - down_value_in_sati_satoshis;

        // Calculate the new allocation sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            ephemeral_contract_shadow_space.allocs_sum - down_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(ShadowAllocDownError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                existing_contract_balance_in_satoshis,
            ));
        }

        // Insert (or update) the account shadow allocation value into the ephemeral states.
        ephemeral_contract_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // Update the contract shadow allocation sum value.
        ephemeral_contract_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Proportionaly increases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn shadow_alloc_up_all(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<u64, ShadowAllocUpAllError> {
        // Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 = (up_value_in_satoshis as u128) * 100_000_000;

        // Get the old contract balance and allocs sum before any mutable borrows.
        let contract_balance_in_satoshis: u64 = self.get_contract_balance(contract_id).ok_or(
            ShadowAllocUpAllError::UnableToGetContractBalance(contract_id),
        )?;

        // Get the old contract allocs sum before any mutable borrows.
        let existing_contract_allocs_sum_in_satoshis: u64 =
            self.get_contract_allocs_sum(contract_id).ok_or(
                ShadowAllocUpAllError::UnableToGetContractAllocsSum(contract_id),
            )?;

        // Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(ShadowAllocUpAllError::OperationNotPossibleWithZeroAllocsSum(contract_id));
        }

        // Calculate the new contract allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            existing_contract_allocs_sum_in_satoshis + up_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(ShadowAllocUpAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // Convert the old contract allocs sum to sati-satoshi value.
        let existing_contract_allocs_sum_in_satisatoshis: u128 =
            (existing_contract_allocs_sum_in_satoshis as u128) * 100_000_000;

        // Initialize a list of update values of individual accounts.
        let mut individual_update_values_in_sati_satoshis: HashMap<ACCOUNT_KEY, u128> =
            HashMap::new();

        // Iterate over all all account in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                // First try the ephemeral shadow space.
                Some(shadow_space) => shadow_space.allocs.iter(),
                // Otherwise from the in-memory shadow space.
                None => self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ShadowAllocUpAllError::ShadowSpaceNotFound(contract_id))?
                    .shadow_space
                    .allocs
                    .iter(),
            }
        {
            // shadow_alloc_value_in_sati_satoshis divided by existing_contract_allocs_sum_in_satisatoshis = x divided by up_value_in_sati_satoshis.
            // NOTE: if the account is ephemerally deallocated, since it's allocation value had to be zero, this will also be zero.
            let individual_up_value_in_sati_satoshis: u128 = (shadow_alloc_value_in_sati_satoshis
                * up_value_in_sati_satoshis)
                / existing_contract_allocs_sum_in_satisatoshis;

            // If the individual up value is greater than zero, insert it into the list of new values.
            if individual_up_value_in_sati_satoshis > 0 {
                // Calculate the new value.
                let individual_new_value_in_sati_satoshis: u128 =
                    shadow_alloc_value_in_sati_satoshis + individual_up_value_in_sati_satoshis;

                // Insert the new value into the list of update values.
                individual_update_values_in_sati_satoshis
                    .insert(*account_key, individual_new_value_in_sati_satoshis);
            }
        }

        // Retrieve the mutable shadow space from the ephemeral states.
        let ephemeral_contract_shadow_space =
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                // If the shadow space is already in the ephemeral states, return it.
                Some(shadow_space) => shadow_space,
                // Otherwise, from the permanent in-memory states.
                None => {
                    let contract_body = self
                        .in_memory
                        .get_mut(&contract_id)
                        .ok_or(ShadowAllocUpAllError::ShadowSpaceNotFound(contract_id))?;

                    // Get the mutable shadow space.
                    let shadow_space = contract_body.shadow_space.clone();

                    // Insert the shadow space into the ephemeral states.
                    self.ephemeral_shadow_spaces
                        .insert(contract_id, shadow_space);

                    // Get the mutable shadow space from the epheremal that we just inserted.
                    let ephemeral_shadow_space = self
                        .ephemeral_shadow_spaces
                        .get_mut(&contract_id)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the shadow space.
                    ephemeral_shadow_space
                }
            };

        // Insert the individual up values into the ephemeral shadow space.
        for (account_key, individual_update_value_in_sati_satoshis) in
            individual_update_values_in_sati_satoshis.iter()
        {
            // Insert the new value into the ephemeral shadow space.
            ephemeral_contract_shadow_space
                .allocs
                .insert(*account_key, *individual_update_value_in_sati_satoshis);
        }

        // Update the allocs sum value in the ephemeral shadow space.
        ephemeral_contract_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Return the number of updated accounts.
        Ok(individual_update_values_in_sati_satoshis.len() as u64)
    }

    /// Proportionaly decreases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `save_all` function.
    pub fn shadow_alloc_down_all(
        &mut self,
        contract_id: [u8; 32],
        down_value_in_satoshis: u64,
    ) -> Result<u64, ShadowAllocDownAllError> {
        // Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 = (down_value_in_satoshis as u128) * 100_000_000;

        // Get the old contract balance and allocs sum before any mutable borrows.
        let contract_balance_in_satoshis: u64 = self.get_contract_balance(contract_id).ok_or(
            ShadowAllocDownAllError::UnableToGetContractBalance(contract_id),
        )?;

        // Get the old contract allocs sum before any mutable borrows.
        let existing_contract_allocs_sum_in_satoshis: u64 =
            self.get_contract_allocs_sum(contract_id).ok_or(
                ShadowAllocDownAllError::UnableToGetContractAllocsSum(contract_id),
            )?;

        // Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(
                ShadowAllocDownAllError::OperationNotPossibleWithZeroAllocsSum(contract_id),
            );
        }

        // Check if would go below zero.
        if down_value_in_satoshis > existing_contract_allocs_sum_in_satoshis {
            return Err(ShadowAllocDownAllError::AllocsSumWouldGoBelowZero(
                contract_id,
                existing_contract_allocs_sum_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new contract allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            existing_contract_allocs_sum_in_satoshis - down_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(ShadowAllocDownAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // Convert the old contract allocs sum to sati-satoshi value.
        let existing_contract_allocs_sum_in_satisatoshis: u128 =
            (existing_contract_allocs_sum_in_satoshis as u128) * 100_000_000;

        // Initialize a list of update values of individual accounts.
        let mut individual_update_values_in_sati_satoshis: HashMap<ACCOUNT_KEY, u128> =
            HashMap::new();

        // Iterate over all all account in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                // First try the ephemeral shadow space.
                Some(shadow_space) => shadow_space.allocs.iter(),
                // Otherwise from the in-memory shadow space.
                None => self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ShadowAllocDownAllError::ShadowSpaceNotFound(contract_id))?
                    .shadow_space
                    .allocs
                    .iter(),
            }
        {
            // shadow_alloc_value_in_sati_satoshis divided by existing_contract_allocs_sum_in_satisatoshis = x divided by down_value_in_sati_satoshis.
            // NOTE: if the account is ephemerally deallocated, since it's allocation value had to be zero, this will also be zero.
            let individual_down_value_in_sati_satoshis: u128 = (shadow_alloc_value_in_sati_satoshis
                * down_value_in_sati_satoshis)
                / existing_contract_allocs_sum_in_satisatoshis;

            // Check if the individual down value would go below zero.
            if individual_down_value_in_sati_satoshis > *shadow_alloc_value_in_sati_satoshis {
                return Err(
                    ShadowAllocDownAllError::IndividualAllocationWouldGoBelowZero(
                        contract_id,
                        *account_key,
                        *shadow_alloc_value_in_sati_satoshis,
                        individual_down_value_in_sati_satoshis,
                    ),
                );
            }

            // If the individual up value is greater than zero, insert it into the list of new values.
            if individual_down_value_in_sati_satoshis > 0 {
                // Calculate the new value.
                let individual_new_value_in_sati_satoshis: u128 =
                    shadow_alloc_value_in_sati_satoshis - individual_down_value_in_sati_satoshis;

                // Insert the new value into the list of update values.
                individual_update_values_in_sati_satoshis
                    .insert(*account_key, individual_new_value_in_sati_satoshis);
            }
        }

        // Retrieve the mutable shadow space from the ephemeral states.
        let ephemeral_contract_shadow_space =
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                // If the shadow space is already in the ephemeral states, return it.
                Some(shadow_space) => shadow_space,
                // Otherwise, from the permanent in-memory states.
                None => {
                    let contract_body = self
                        .in_memory
                        .get_mut(&contract_id)
                        .ok_or(ShadowAllocDownAllError::ShadowSpaceNotFound(contract_id))?;

                    // Get the mutable shadow space.
                    let shadow_space = contract_body.shadow_space.clone();

                    // Insert the shadow space into the ephemeral states.
                    self.ephemeral_shadow_spaces
                        .insert(contract_id, shadow_space);

                    // Get the mutable shadow space from the epheremal that we just inserted.
                    let ephemeral_shadow_space = self
                        .ephemeral_shadow_spaces
                        .get_mut(&contract_id)
                        .expect("This cannot happen because we just inserted it.");

                    // Return the shadow space.
                    ephemeral_shadow_space
                }
            };

        // Insert the individual up values into the ephemeral shadow space.
        for (account_key, individual_update_value_in_sati_satoshis) in
            individual_update_values_in_sati_satoshis.iter()
        {
            // Insert the new value into the ephemeral shadow space.
            ephemeral_contract_shadow_space
                .allocs
                .insert(*account_key, *individual_update_value_in_sati_satoshis);
        }

        // Update the allocs sum value in the ephemeral shadow space.
        ephemeral_contract_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Return the number of updated accounts.
        Ok(individual_update_values_in_sati_satoshis.len() as u64)
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        // Restore the ephemeral states from the backup.
        self.restore_ephemeral_states();
    }

    /// Clears all epheremal changes.
    pub fn rollback_all(&mut self) {
        // Clear the ephemeral states.
        self.ephemeral_balances.clear();
        self.ephemeral_shadow_spaces.clear();

        // Clear the ephemeral states backup.
        self.backup_of_ephemeral_balances.clear();
        self.backup_of_ephemeral_shadow_spaces.clear();
    }

    /// Saves all epheremal changes in-memory and on-disk.
    pub fn save_all(&mut self) -> Result<(), ContractCoinHolderSaveError> {
        // #0 Save ephemeral balances.
        for (contract_id, ephemeral_contract_balance) in self.ephemeral_balances.iter() {
            // #0.0 In-memory insertion.
            {
                // Get mutable in-memory permanent contract body.
                let in_memory_permanent_contract_body = self.in_memory.get_mut(contract_id).ok_or(
                    ContractCoinHolderSaveError::ContractBodyNotFound(*contract_id),
                )?;

                // Update the balance in the in-memory states.
                in_memory_permanent_contract_body.balance = *ephemeral_contract_balance;
            }

            // #0.1 On-disk insertion.
            {
                // Save the balance to the balances db.
                self.balance_db
                    .insert(
                        contract_id,
                        ephemeral_contract_balance.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        ContractCoinHolderSaveError::BalanceValueInsertError(
                            *contract_id,
                            *ephemeral_contract_balance,
                            e,
                        )
                    })?;
            }
        }

        // #1 Save ephemeral shadow spaces.
        for (contract_id, ephemeral_shadow_space) in self.ephemeral_shadow_spaces.iter() {
            // #1.0 In-memory insertion.
            {
                // Get mutable in-memory permanent contract body.
                let in_memory_permanent_contract_body = self.in_memory.get_mut(contract_id).ok_or(
                    ContractCoinHolderSaveError::ContractBodyNotFound(*contract_id),
                )?;

                // Update the shadow space in the in-memory permanent states.
                in_memory_permanent_contract_body.shadow_space = ephemeral_shadow_space.clone();
            }

            // #1.1 On-disk insertion.
            {
                // Open the contract tree using the contract ID as the tree name.
                let on_disk_permanent_shadow_space = self
                    .shadow_space_db
                    .open_tree(contract_id)
                    .map_err(|e| ContractCoinHolderSaveError::OpenTreeError(*contract_id, e))?;

                // Insert all shadows into the on-disk contract tree.
                for (ephemeral_shadow_account_key, ephemeral_shadow_alloc_value) in
                    ephemeral_shadow_space.allocs.iter()
                {
                    on_disk_permanent_shadow_space
                        .insert(
                            ephemeral_shadow_account_key.to_vec(),
                            ephemeral_shadow_alloc_value.to_le_bytes().to_vec(),
                        )
                        .map_err(|e| {
                            ContractCoinHolderSaveError::ShadowSpaceTreeAllocInsertError(
                                *contract_id,
                                *ephemeral_shadow_account_key,
                                *ephemeral_shadow_alloc_value,
                                e,
                            )
                        })?;
                }

                // Also save the allocs sum with the special key (0xff..).
                on_disk_permanent_shadow_space
                    .insert(
                        [0xff; 32].to_vec(),
                        ephemeral_shadow_space.allocs_sum.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        ContractCoinHolderSaveError::ShadowSpaceTreeAllocsSumInsertError(
                            *contract_id,
                            ephemeral_shadow_space.allocs_sum,
                            e,
                        )
                    })?;
            }
        }
        // #2 Handle deallocs
        {
            for (contract_id, ephemeral_dealloc_list) in self.epheremal_dealloc_list.iter() {
                // In-memory deletion.
                {
                    // Get mutable in-memory permanent contract body.
                    let in_memory_permanent_contract_body =
                        self.in_memory.get_mut(contract_id).ok_or(
                            ContractCoinHolderSaveError::ContractBodyNotFound(*contract_id),
                        )?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        if in_memory_permanent_contract_body
                            .shadow_space
                            .allocs
                            .remove(account_key)
                            .is_none()
                        {
                            return Err(ContractCoinHolderSaveError::InMemoryDeallocSaveError(
                                *contract_id,
                                *account_key,
                            ));
                        };
                    }
                }

                // On-disk deletion.
                {
                    // Open the contract tree using the contract ID as the tree name.
                    let on_disk_permanent_shadow_space = self
                        .shadow_space_db
                        .open_tree(contract_id)
                        .map_err(|e| ContractCoinHolderSaveError::OpenTreeError(*contract_id, e))?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        match on_disk_permanent_shadow_space.remove(account_key) {
                            Ok(_) => (),
                            Err(err) => {
                                return Err(ContractCoinHolderSaveError::OnDiskDeallocSaveError(
                                    *contract_id,
                                    *account_key,
                                    err,
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Clear the ephemeral states.
        self.ephemeral_balances.clear();
        self.ephemeral_shadow_spaces.clear();
        self.epheremal_dealloc_list.clear();

        // Clear the ephemeral states backup.
        self.backup_of_ephemeral_balances.clear();
        self.backup_of_ephemeral_shadow_spaces.clear();
        self.backup_of_ephemeral_dealloc_list.clear();

        Ok(())
    }
}

/// Erase by db path.
pub fn erase_contract_coin_holder(chain: Chain) {
    // Balance db path.
    let balance_path = format!("db/{}/coin/contract/balance", chain.to_string());

    // Shadow space db path.
    let shadow_space_path = format!("db/{}/coin/contract/shadow_space", chain.to_string());

    // Erase the paths.
    let _ = std::fs::remove_dir_all(balance_path);
    let _ = std::fs::remove_dir_all(shadow_space_path);
}
