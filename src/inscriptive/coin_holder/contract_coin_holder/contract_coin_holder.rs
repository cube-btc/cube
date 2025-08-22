use super::contract_coin_holder_error::{
    ContractCoinHolderConstructionError, ContractCoinHolderSaveError,
};
use crate::inscriptive::coin_holder::contract_coin_holder::contract_coin_holder_error::{
    ContractCoinHolderRegisterError, ShadowAllocDownError, ShadowAllocError, ShadowAllocUpError,
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

/// A struct for representing a shadow space of a contract.
#[derive(Clone)]
struct ContractShadowSpace {
    // Total allocated BTC value of the entire shadow space.
    pub allocs_sum: u64,
    // Allocated BTC values of each account.
    pub allocs: HashMap<ACCOUNT_KEY, u64>,
}

/// A struct for containing BTC balance and shadow space allocations of a contract.
#[derive(Clone)]
struct ContractBody {
    // Contract's BTC balance.
    pub balance: u64,
    // Contract's shadow space.
    pub shadow_space: ContractShadowSpace,
}

/// A struct for containing contract/program states in-memory and on-disk.
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

    // BACKUPS
    /// In-memory cache of ephemeral contract balances backup.
    backup_of_ephemeral_balances: HashMap<CONTRACT_ID, u64>,
    /// In-memory cache of ephemeral contract shadow spaces backup.
    backup_of_ephemeral_shadow_spaces: HashMap<CONTRACT_ID, ContractShadowSpace>,
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
            let contract_id: CONTRACT_ID = tree_name.as_ref().try_into().map_err(|_| {
                ContractCoinHolderConstructionError::InvalidContractIDBytes(tree_name.to_vec())
            })?;

            // Open the contract shadow space tree.
            let shadow_space_tree = shadow_space_db.open_tree(&tree_name).map_err(|e| {
                ContractCoinHolderConstructionError::ShadowSpaceTreeOpenError(contract_id, e)
            })?;

            // Initialize the in-memory cache of contract shadow space.
            let mut allocs = HashMap::<ACCOUNT_KEY, u64>::new();

            // Calculated allocs sum.
            let mut calculated_allocs_sum: u64 = 0;

            // On-disk stored allocs sum.
            let mut stored_allocs_sum: u64 = 0;

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

                // Convert the value to an allocation value.
                let alloc_value: u64 =
                    u64::from_le_bytes(alloc_value.as_ref().try_into().map_err(|_| {
                        ContractCoinHolderConstructionError::InvalidShadowBalance(
                            alloc_value.to_vec(),
                        )
                    })?);

                // If the account key is the special key (0xff..),
                // this represents the allocs sum in the shadow space.
                if alloc_account_key == [0xff; 32] {
                    stored_allocs_sum = alloc_value;
                    continue;
                }

                // Update the shadow space allocations.
                allocs.insert(alloc_account_key, alloc_value);
                calculated_allocs_sum += alloc_value; // Update the shadow space allocations sum.
            }

            // Check if the calculated allocs sum matches the stored allocs sum.
            if calculated_allocs_sum != stored_allocs_sum {
                return Err(ContractCoinHolderConstructionError::AllocsSumMismatch(
                    contract_id,
                    calculated_allocs_sum,
                    stored_allocs_sum,
                ));
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
            if calculated_allocs_sum > contract_balance {
                return Err(
                    ContractCoinHolderConstructionError::AllocsSumExceedsTheContractBalance(
                        contract_id,
                        stored_allocs_sum,
                        contract_balance,
                    ),
                );
            }

            // Create the contract body.
            let contract_body = ContractBody {
                balance: contract_balance,
                shadow_space: ContractShadowSpace {
                    allocs_sum: stored_allocs_sum,
                    allocs,
                },
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
            backup_of_ephemeral_balances: HashMap::<CONTRACT_ID, u64>::new(),
            backup_of_ephemeral_shadow_spaces: HashMap::<CONTRACT_ID, ContractShadowSpace>::new(),
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
    }

    /// Restores ephemeral states from the backup.
    fn restore_ephemeral_states(&mut self) {
        self.ephemeral_balances = self.backup_of_ephemeral_balances.clone();
        self.ephemeral_shadow_spaces = self.backup_of_ephemeral_shadow_spaces.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the ephemeral states.
        self.backup_ephemeral_states();
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

    /// Get the shadow allocation value of an account for a specific contract ID.
    pub fn get_account_shadow_alloc_value(
        &self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some(shadow_space) = self.ephemeral_shadow_spaces.get(&contract_id) {
            return shadow_space.allocs.get(&account_key).cloned();
        }

        // And then try to get from the in-memory states.
        self.in_memory
            .get(&contract_id)
            .and_then(|body| body.shadow_space.allocs.get(&account_key).cloned())
    }

    /// Registers a list of contract IDs.
    pub fn register_contract_ids(
        &mut self,
        contract_ids: Vec<[u8; 32]>,
    ) -> Result<(), ContractCoinHolderRegisterError> {
        // Check if at least one of the contract IDs is already registered.
        for contract_id in contract_ids.iter() {
            if self.in_memory.contains_key(contract_id) {
                return Err(
                    ContractCoinHolderRegisterError::ContractIDAlreadyRegistered(*contract_id),
                );
            }
        }

        // Iterate over all contract IDs.
        for contract_id in contract_ids {
            // Save in-memory states.
            {
                // Get the contract balance.
                let fresh_new_contract_body = ContractBody {
                    balance: 0,
                    shadow_space: ContractShadowSpace {
                        allocs_sum: 0,
                        allocs: HashMap::new(),
                    },
                };

                // Insert the contract body into the in-memory cache.
                self.in_memory.insert(contract_id, fresh_new_contract_body);
            }

            // Save on-disk states.
            {
                // Balances-db.
                {
                    // Create the value bytes.
                    let value_bytes: [u8; 8] = 0u64.to_le_bytes();

                    // Save the balance to the balances db.
                    self.balance_db
                        .insert(contract_id, value_bytes.to_vec())
                        .map_err(|e| {
                            ContractCoinHolderRegisterError::TreeValueInsertError(contract_id, e)
                        })?;
                }

                // Shadow-space-db.
                {
                    // Only insert the allocs sum with the special key (0xff..) into a new tree.
                    let shadow_space_tree =
                        self.shadow_space_db.open_tree(contract_id).map_err(|e| {
                            ContractCoinHolderRegisterError::OpenTreeError(contract_id, e)
                        })?;

                    // Create the value bytes.
                    let value_bytes: [u8; 8] = 0u64.to_le_bytes();

                    // Insert the allocs sum with the special key (0xff..).
                    shadow_space_tree
                        .insert([0xff; 32].to_vec(), value_bytes.to_vec())
                        .map_err(|e| {
                            ContractCoinHolderRegisterError::TreeValueInsertError(contract_id, e)
                        })?;
                }
            }
        }

        // Return the result.
        Ok(())
    }

    // Creates an allocation space for an account in a contract shadow space.
    pub fn shadow_alloc(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Result<(), ShadowAllocError> {
        // Check if the account key is already allocated.
        if self
            .get_account_shadow_alloc_value(contract_id, account_key)
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

    /// Inserts or updates a shadow allocation value by key and contract ID ephemerally.
    pub fn shadow_alloc_up(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
        increase_value: u64,
    ) -> Result<(), ShadowAllocUpError> {
        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value = self
            .get_account_shadow_alloc_value(contract_id, account_key)
            .ok_or(ShadowAllocUpError::UnableToGetOldAccountAllocValue(
                contract_id,
                account_key,
            ))?;

        let existing_contract_balance = self
            .get_contract_balance(contract_id)
            .ok_or(ShadowAllocUpError::UnableToGetContractBalance(contract_id))?;

        // Try to get the shadow space from ephemeral states first.
        let ephemeral_contract_shadow_space =
            match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
                Some(shadow_space) => shadow_space,
                None => {
                    // Otherwise, from the permanent in-memory states.
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
        let new_account_shadow_alloc_value = existing_account_shadow_alloc_value + increase_value;

        // Calculate the new allocation sum value.
        let new_contract_allocs_sum_value =
            ephemeral_contract_shadow_space.allocs_sum + increase_value;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value > existing_contract_balance {
            return Err(ShadowAllocUpError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value,
                existing_contract_balance,
            ));
        }

        // Insert (or update) the account shadow allocation value into the ephemeral states.
        ephemeral_contract_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value);

        // Update the contract shadow allocation sum value.
        ephemeral_contract_shadow_space.allocs_sum = new_contract_allocs_sum_value;

        // Return the result.
        Ok(())
    }

    /// Decreases a shadow allocation value by key and contract ID ephemerally.
    pub fn shadow_alloc_down(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
        decrease_value: u64,
    ) -> Result<(), ShadowAllocDownError> {
        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value = self
            .get_account_shadow_alloc_value(contract_id, account_key)
            .ok_or(ShadowAllocDownError::UnableToGetOldAccountAllocValue(
                contract_id,
                account_key,
            ))?;

        let existing_contract_balance = self.get_contract_balance(contract_id).ok_or(
            ShadowAllocDownError::UnableToGetContractBalance(contract_id),
        )?;

        // Check if the decrease would make the allocation value go below zero.
        if decrease_value > existing_account_shadow_alloc_value {
            return Err(ShadowAllocDownError::AllocValueWouldGoBelowZero(
                contract_id,
                account_key,
                existing_account_shadow_alloc_value,
                decrease_value,
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
        let new_account_shadow_alloc_value = existing_account_shadow_alloc_value - decrease_value;

        // Calculate the new allocation sum value.
        let new_contract_allocs_sum_value =
            ephemeral_contract_shadow_space.allocs_sum - decrease_value;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value > existing_contract_balance {
            return Err(ShadowAllocDownError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value,
                existing_contract_balance,
            ));
        }

        // Insert (or update) the account shadow allocation value into the ephemeral states.
        ephemeral_contract_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value);

        // Update the contract shadow allocation sum value.
        ephemeral_contract_shadow_space.allocs_sum = new_contract_allocs_sum_value;

        // Return the result.
        Ok(())
    }

    /// Reverts the state update(s) associated with the last execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn rollback_last(&mut self) {
        // Restore the ephemeral states from the backup.
        self.restore_ephemeral_states();
    }

    /// Reverts all state updates associated with all executions.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn rollback_all(&mut self) {
        // Clear the ephemeral states.
        self.ephemeral_balances.clear();
        self.ephemeral_shadow_spaces.clear();

        // Clear the ephemeral states backup.
        self.backup_of_ephemeral_balances.clear();
        self.backup_of_ephemeral_shadow_spaces.clear();
    }

    /// Saves the states updated associated with all executions (on-disk and in-memory).
    ///
    /// TODO Performance Optimization: Open the tree *once per contract ID* and then insert all key-values at once.
    pub fn save_all_executions(&mut self) -> Result<(), ContractCoinHolderSaveError> {
        // #0 Save ephemeral balances.
        for (contract_id, ephemeral_balance) in self.ephemeral_balances.iter() {
            // #0.0 In-memory insertion.
            {
                // Get mutable contract body.
                let contract_body = self.in_memory.get_mut(contract_id).ok_or(
                    ContractCoinHolderSaveError::ContractBodyNotFound(*contract_id),
                )?;

                // Update the balance in the in-memory states.
                contract_body.balance = *ephemeral_balance;
            }

            // #0.1 On-disk insertion.
            {
                // Save the balance to the balances db.
                self.balance_db
                    .insert(contract_id, ephemeral_balance.to_le_bytes().to_vec())
                    .map_err(|e| {
                        ContractCoinHolderSaveError::TreeValueInsertError(
                            *contract_id,
                            *contract_id,
                            *ephemeral_balance,
                            e,
                        )
                    })?;
            }
        }

        // #1 Save ephemeral shadow spaces.
        for (contract_id, ephemeral_shadow_space) in self.ephemeral_shadow_spaces.iter() {
            // #1.0 In-memory insertion.
            {
                // Get mutable contract body.
                let contract_body = self.in_memory.get_mut(contract_id).ok_or(
                    ContractCoinHolderSaveError::ContractBodyNotFound(*contract_id),
                )?;

                // Update the shadow space in the in-memory states.
                contract_body.shadow_space = ephemeral_shadow_space.clone();
            }

            // #1.1 On-disk insertion.
            {
                // Save the shadows to the shadows db.
                let shadow_space_tree = self
                    .shadow_space_db
                    .open_tree(contract_id)
                    .map_err(|e| ContractCoinHolderSaveError::OpenTreeError(*contract_id, e))?;

                // Insert all shadows into the on-disk contract tree.
                for (ephemeral_shadow_account_key, ephemeral_shadow_value) in
                    ephemeral_shadow_space.allocs.iter()
                {
                    shadow_space_tree
                        .insert(
                            ephemeral_shadow_account_key.to_vec(),
                            ephemeral_shadow_value.to_le_bytes().to_vec(),
                        )
                        .map_err(|e| {
                            ContractCoinHolderSaveError::TreeValueInsertError(
                                *contract_id,
                                *ephemeral_shadow_account_key,
                                *ephemeral_shadow_value,
                                e,
                            )
                        })?;
                }

                // Also save the allocs sum with the special key (0xff..).
                shadow_space_tree
                    .insert(
                        [0xff; 32].to_vec(),
                        ephemeral_shadow_space.allocs_sum.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        ContractCoinHolderSaveError::TreeValueInsertError(
                            *contract_id,
                            [0xff; 32],
                            ephemeral_shadow_space.allocs_sum,
                            e,
                        )
                    })?;
            }
        }

        // Clear the ephemeral states.
        self.ephemeral_balances.clear();
        self.ephemeral_shadow_spaces.clear();

        // Clear the ephemeral states backup.
        self.backup_of_ephemeral_balances.clear();
        self.backup_of_ephemeral_shadow_spaces.clear();

        Ok(())
    }
}
