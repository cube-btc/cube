use super::contract_coin_holder_error::{
    ContractCoinHolderConstructionError, ContractCoinHolderSaveError,
};
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// BTC balance of a contract in satoshis.
#[allow(non_camel_case_types)]
type CONTRACT_BALANCE = u64;

/// Total BTC shadow allocation of a contract in satoshis.
#[allow(non_camel_case_types)]
type CONTRACT_SHADOW_ALLOC_SUM = u64;

/// Shadow account key.
#[allow(non_camel_case_types)]
type SHADOW_ACCOUNT_KEY = [u8; 32];

/// Shadow account coin balance.
#[allow(non_camel_case_types)]
type SHADOW_ACCOUNT_COIN_BALANCE = u64;

/// Shadow space of a contract.
#[allow(non_camel_case_types)]
type CONTRACT_SHADOW_SPACE = HashMap<SHADOW_ACCOUNT_KEY, SHADOW_ACCOUNT_COIN_BALANCE>;

/// A struct for containing contract/program states in-memory and on-disk.
pub struct ContractCoinHolder {
    // IN-MEMORY STATES
    /// In-memory cache of states: CONTRACT_ID -> (CONTRACT_BALANCE, CONTRACT_SHADOW_ALLOC_SUM, CONTRACT_SHADOW_SPACE)
    coins: HashMap<
        CONTRACT_ID,
        (
            CONTRACT_BALANCE,
            CONTRACT_SHADOW_ALLOC_SUM,
            CONTRACT_SHADOW_SPACE,
        ),
    >,

    // ON-DISK STATES
    /// Sled DB with all contract balances in a single tree.
    balance_db: sled::Db,
    /// Sled DB with all contract shadow allocations in a single tree.
    shadow_alloc_sum_db: sled::Db,
    /// Sled DB with contract shadow trees.
    shadow_space_db: sled::Db,

    // EPHEMERAL STATES
    /// In-memory cache of ephemeral balances.
    ephemeral_balances: HashMap<CONTRACT_ID, CONTRACT_BALANCE>,
    /// In-memory cache of ephemeral shadow allocations.
    ephemeral_shadow_allocs: HashMap<CONTRACT_ID, CONTRACT_SHADOW_ALLOC_SUM>,
    /// In-memory cache of ephemeral shadow spaces.
    ephemeral_shadow_spaces: HashMap<CONTRACT_ID, CONTRACT_SHADOW_SPACE>,

    // BACKUPS
    /// In-memory cache of ephemeral states backup.
    ephemeral_balances_backup: HashMap<CONTRACT_ID, CONTRACT_BALANCE>,
    /// In-memory cache of ephemeral shadow allocations backup.
    ephemeral_shadow_allocs_backup: HashMap<CONTRACT_ID, CONTRACT_SHADOW_ALLOC_SUM>,
    /// In-memory cache of ephemeral shadow spaces backup.
    ephemeral_shadow_spaces_backup: HashMap<CONTRACT_ID, CONTRACT_SHADOW_SPACE>,
}

/// Guarded state holder.
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

        // Open the shadow allocation db.
        let shadow_alloc_sum_path =
            format!("db/{}/coin/contract/shadow_alloc_sum", chain.to_string());
        let shadow_alloc_sum_db = sled::open(shadow_alloc_sum_path)
            .map_err(ContractCoinHolderConstructionError::ShadowAllocDBOpenError)?;

        // Open the shadows db.
        let shadow_space_path = format!("db/{}/coin/contract/shadow_space", chain.to_string());
        let shadow_space_db = sled::open(shadow_space_path)
            .map_err(ContractCoinHolderConstructionError::ShadowSpaceDBOpenError)?;

        // Initialize the in-memory cache of contract coins.
        let mut coins = HashMap::<
            CONTRACT_ID,
            (
                CONTRACT_BALANCE,
                CONTRACT_SHADOW_ALLOC_SUM,
                CONTRACT_SHADOW_SPACE,
            ),
        >::new();

        // Iterate over all contract trees in the shadows db.
        for tree_name in shadow_space_db.tree_names() {
            let contract_id: CONTRACT_ID = tree_name.as_ref().try_into().map_err(|_| {
                ContractCoinHolderConstructionError::InvalidContractIDBytes(tree_name.to_vec())
            })?;

            // Open the contract tree.
            let tree = shadow_space_db.open_tree(&tree_name).map_err(|e| {
                ContractCoinHolderConstructionError::ShadowSpaceTreeOpenError(contract_id, e)
            })?;

            // Initialize the in-memory cache of contract shadows.
            let mut contract_shadow_space =
                HashMap::<SHADOW_ACCOUNT_KEY, SHADOW_ACCOUNT_COIN_BALANCE>::new();

            // Iterate over all items in the contract tree.
            for contract_shadow in tree.iter() {
                // Get the key and value.
                let (k, v) = match contract_shadow {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(
                            ContractCoinHolderConstructionError::ContractShadowIterError(e),
                        );
                    }
                };

                // Convert the key to an allocation account key.
                let shadow_account_key: [u8; 32] = k.to_vec().try_into().map_err(|_| {
                    ContractCoinHolderConstructionError::InvalidShadowAccountKey(k.to_vec())
                })?;

                // Convert the value to an allocation value.
                let shadow_value: u64 =
                    u64::from_le_bytes(v.as_ref().try_into().map_err(|_| {
                        ContractCoinHolderConstructionError::InvalidShadowBalance(v.to_vec())
                    })?);

                // Insert the shadow value into the shadow space.
                contract_shadow_space.insert(shadow_account_key, shadow_value);
            }

            // Get the contract balance from the balances db.
            let contract_balance = {
                // Get the balance value using contract_id as the key
                match balance_db.get(&contract_id) {
                    Ok(Some(balance_bytes)) => {
                        u64::from_le_bytes(balance_bytes.as_ref().try_into().map_err(|_| {
                            ContractCoinHolderConstructionError::InvalidContractBalance(
                                balance_bytes.to_vec(),
                            )
                        })?)
                    }
                    Ok(None) => 0, // Default balance if not found
                    Err(e) => {
                        return Err(ContractCoinHolderConstructionError::BalanceGetError(
                            contract_id,
                            e,
                        ));
                    }
                }
            };

            // Get the contract shadow allocation from the shadow allocation db.
            let contract_shadow_alloc_sum = {
                // Get the shadow allocation value using contract_id as the key
                match shadow_alloc_sum_db.get(&contract_id) {
                    Ok(Some(shadow_alloc_bytes)) => u64::from_le_bytes(
                        shadow_alloc_bytes.as_ref().try_into().map_err(|_| {
                            ContractCoinHolderConstructionError::InvalidShadowAllocation(
                                shadow_alloc_bytes.to_vec(),
                            )
                        })?,
                    ),
                    Ok(None) => 0, // Default shadow allocation if not found
                    Err(e) => {
                        return Err(
                            ContractCoinHolderConstructionError::ShadowAllocationGetError(
                                contract_id,
                                e,
                            ),
                        );
                    }
                }
            };

            // Insert the contract shadows, shadow allocation, and balance into the in-memory cache.
            coins.insert(
                contract_id,
                (
                    contract_balance,
                    contract_shadow_alloc_sum,
                    contract_shadow_space,
                ),
            );
        }

        // Create the state holder.
        let contract_coin_holder = ContractCoinHolder {
            coins,
            balance_db,
            shadow_alloc_sum_db,
            shadow_space_db,
            ephemeral_balances: HashMap::<CONTRACT_ID, CONTRACT_BALANCE>::new(),
            ephemeral_shadow_allocs: HashMap::<CONTRACT_ID, CONTRACT_SHADOW_ALLOC_SUM>::new(),
            ephemeral_shadow_spaces: HashMap::<CONTRACT_ID, CONTRACT_SHADOW_SPACE>::new(),
            ephemeral_balances_backup: HashMap::<CONTRACT_ID, CONTRACT_BALANCE>::new(),
            ephemeral_shadow_allocs_backup: HashMap::<CONTRACT_ID, CONTRACT_SHADOW_ALLOC_SUM>::new(
            ),
            ephemeral_shadow_spaces_backup: HashMap::<CONTRACT_ID, CONTRACT_SHADOW_SPACE>::new(),
        };

        // Return the guarded state holder.
        Ok(Arc::new(Mutex::new(contract_coin_holder)))
    }

    /// Clones ephemeral states into the backup.
    fn backup_ephemeral_states(&mut self) {
        self.ephemeral_balances_backup = self.ephemeral_balances.clone();
        self.ephemeral_shadow_allocs_backup = self.ephemeral_shadow_allocs.clone();
        self.ephemeral_shadow_spaces_backup = self.ephemeral_shadow_spaces.clone();
    }

    /// Restores ephemeral states from the backup.
    fn restore_ephemeral_states(&mut self) {
        self.ephemeral_balances = self.ephemeral_balances_backup.clone();
        self.ephemeral_shadow_allocs = self.ephemeral_shadow_allocs_backup.clone();
        self.ephemeral_shadow_spaces = self.ephemeral_shadow_spaces_backup.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the ephemeral states.
        self.backup_ephemeral_states();
    }

    /// Get the contract coin balance for a contract ID.
    pub async fn get_contract_balance(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some(balance) = self.ephemeral_balances.get(&contract_id) {
            return Some(*balance);
        }

        // And then try to get from the states.
        self.coins.get(&contract_id).map(|(balance, _, _)| *balance)
    }

    /// Get the contract shadow allocation for a contract ID.
    pub async fn get_contract_shadow_alloc_sum(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some(shadow_alloc) = self.ephemeral_shadow_allocs.get(&contract_id) {
            return Some(*shadow_alloc);
        }

        // And then try to get from the states.
        self.coins
            .get(&contract_id)
            .map(|(_, shadow_alloc, _)| *shadow_alloc)
    }

    /// Get the shadow balance of an account for a specific contract ID.
    pub async fn get_shadow_balance_of_an_account_for_a_contract(
        &self,
        contract_id: [u8; 32],
        shadow_account_key: SHADOW_ACCOUNT_KEY,
    ) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some(shadow_balance) = self.ephemeral_shadow_spaces.get(&contract_id) {
            return shadow_balance.get(&shadow_account_key).cloned();
        }

        // And then try to get from the states.
        self.coins
            .get(&contract_id)
            .and_then(|(_, _, shadow_balance)| shadow_balance.get(&shadow_account_key).cloned())
    }

    /// Inserts or updates a shadow balance by key and contract ID ephemerally.
    pub async fn insert_update_account_shadow_balance(
        &mut self,
        contract_id: [u8; 32],
        shadow_account_key: SHADOW_ACCOUNT_KEY,
        shadow_value: SHADOW_ACCOUNT_COIN_BALANCE,
    ) -> Option<()> {
        // Get mutable ephemeral shadow spaces.
        let ephemeral_shadow_balance = match self.ephemeral_shadow_spaces.get_mut(&contract_id) {
            Some(shadow_balance) => shadow_balance,
            None => {
                // Create it if it doesn't exist.
                let new_shadow_balance =
                    HashMap::<SHADOW_ACCOUNT_KEY, SHADOW_ACCOUNT_COIN_BALANCE>::new();

                // Insert it.
                self.ephemeral_shadow_spaces
                    .insert(contract_id, new_shadow_balance);

                // Get it again.
                self.ephemeral_shadow_spaces.get_mut(&contract_id).unwrap() // Safe because we just inserted it.
            }
        };

        // Insert (or update) the value into the ephemeral states.
        ephemeral_shadow_balance.insert(shadow_account_key, shadow_value);

        // Return the result.
        Some(())
    }

    /// Updates the contract coin balance for a contract ID ephemerally.
    pub async fn update_contract_balance(
        &mut self,
        contract_id: [u8; 32],
        new_balance: CONTRACT_BALANCE,
    ) -> Option<()> {
        // Get mutable ephemeral states.
        let ephemeral_balance = match self.ephemeral_balances.get_mut(&contract_id) {
            Some(balance) => balance,
            None => {
                // Create it if it doesn't exist.
                let new_balance = 0;

                // Insert it.
                self.ephemeral_balances.insert(contract_id, new_balance);

                // Get it again.
                self.ephemeral_balances.get_mut(&contract_id).unwrap() // Safe because we just inserted it.
            }
        };

        // Update the balance in the ephemeral states.
        *ephemeral_balance = new_balance;

        // Return the result.
        Some(())
    }

    /// Updates the contract shadow allocation for a contract ID ephemerally.
    pub async fn update_contract_shadow_alloc_sum(
        &mut self,
        contract_id: [u8; 32],
        new_shadow_alloc: CONTRACT_SHADOW_ALLOC_SUM,
    ) -> Option<()> {
        // Get mutable ephemeral states.
        let ephemeral_shadow_alloc = match self.ephemeral_shadow_allocs.get_mut(&contract_id) {
            Some(shadow_alloc) => shadow_alloc,
            None => {
                // Create it if it doesn't exist.
                let new_shadow_alloc = 0;

                // Insert it.
                self.ephemeral_shadow_allocs
                    .insert(contract_id, new_shadow_alloc);

                // Get it again.
                self.ephemeral_shadow_allocs.get_mut(&contract_id).unwrap() // Safe because we just inserted it.
            }
        };

        // Update the shadow allocation in the ephemeral states.
        *ephemeral_shadow_alloc = new_shadow_alloc;

        // Return the result.
        Some(())
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
        self.ephemeral_shadow_allocs.clear();
        self.ephemeral_shadow_spaces.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_balances_backup.clear();
        self.ephemeral_shadow_allocs_backup.clear();
        self.ephemeral_shadow_spaces_backup.clear();
    }

    /// Saves the states updated associated with all executions (on-disk and in-memory).
    ///
    /// TODO Performance Optimization: Open the tree *once per contract ID* and then insert all key-values at once.
    pub fn save_all_executions(&mut self) -> Result<(), ContractCoinHolderSaveError> {
        // Save ephemeral balances and shadow allocations.
        for (contract_id, ephemeral_balance) in self.ephemeral_balances.iter() {
            // In-memory insertion.
            {
                // Insert or update the balance and shadow allocation in memory.
                self.coins.insert(
                    *contract_id,
                    (
                        *ephemeral_balance,
                        self.ephemeral_shadow_allocs
                            .get(contract_id)
                            .cloned()
                            .unwrap_or_default(),
                        self.ephemeral_shadow_spaces
                            .get(contract_id)
                            .cloned()
                            .unwrap_or_default(),
                    ),
                );
            }

            // On-disk insertion.
            {
                // Save the balance to the balances db.
                self.balance_db
                    .insert(contract_id, ephemeral_balance.to_le_bytes().to_vec())
                    .map_err(|e| {
                        ContractCoinHolderSaveError::TreeValueInsertError(
                            contract_id.to_owned(),
                            contract_id.to_vec(),
                            *ephemeral_balance,
                            e,
                        )
                    })?;
            }
        }

        // Save ephemeral shadow allocation sums to disk and memory.
        for (contract_id, ephemeral_shadow_alloc_sum) in self.ephemeral_shadow_allocs.iter() {
            // In-memory insertion.
            {
                // Insert or update the balance and shadow allocation in memory.
                self.coins.insert(
                    *contract_id,
                    (
                        self.ephemeral_balances
                            .get(contract_id)
                            .cloned()
                            .unwrap_or_default(),
                        *ephemeral_shadow_alloc_sum,
                        self.ephemeral_shadow_spaces
                            .get(contract_id)
                            .cloned()
                            .unwrap_or_default(),
                    ),
                );
            }

            // On-disk insertion.
            {
                // Save the shadow allocation sum to the shadow allocation db.
                self.shadow_alloc_sum_db
                    .insert(
                        contract_id,
                        ephemeral_shadow_alloc_sum.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        ContractCoinHolderSaveError::TreeValueInsertError(
                            contract_id.to_owned(),
                            contract_id.to_vec(),
                            *ephemeral_shadow_alloc_sum,
                            e,
                        )
                    })?;
            }
        }

        // Save ephemeral shadow spaces.
        for (contract_id, ephemeral_shadow_space) in self.ephemeral_shadow_spaces.iter() {
            // In-memory insertion.
            {
                // Get mutable shadow spaces or create if doesn't exist.
                let shadow_spaces =
                    if let Some((_, _, shadow_spaces)) = self.coins.get_mut(contract_id) {
                        shadow_spaces
                    } else {
                        // Create it if it doesn't exist.
                        let new_shadow_spaces =
                            HashMap::<SHADOW_ACCOUNT_KEY, SHADOW_ACCOUNT_COIN_BALANCE>::new();

                        // Insert it.
                        self.coins
                            .insert(contract_id.to_owned(), (0, 0, new_shadow_spaces));

                        // Get it again.
                        &mut self.coins.get_mut(contract_id).unwrap().2 // Safe because we just inserted it.
                    };

                // Iterate over all ephemeral shadow balances for this contract.
                for (ephemeral_shadow_account_key, ephemeral_shadow_value) in
                    ephemeral_shadow_space.iter()
                {
                    shadow_spaces.insert(
                        ephemeral_shadow_account_key.clone(),
                        *ephemeral_shadow_value,
                    );
                }
            }

            // On-disk insertion.
            {
                // Save the shadows to the shadows db.
                let shadow_space_tree =
                    self.shadow_space_db.open_tree(&contract_id).map_err(|e| {
                        ContractCoinHolderSaveError::OpenTreeError(contract_id.to_owned(), e)
                    })?;

                // Insert all shadows into the on-disk contract tree.
                for (ephemeral_shadow_account_key, ephemeral_shadow_value) in
                    ephemeral_shadow_space.iter()
                {
                    shadow_space_tree
                        .insert(
                            ephemeral_shadow_account_key.to_vec(),
                            ephemeral_shadow_value.to_le_bytes().to_vec(),
                        )
                        .map_err(|e| {
                            ContractCoinHolderSaveError::TreeValueInsertError(
                                contract_id.to_owned(),
                                ephemeral_shadow_account_key.to_vec(),
                                *ephemeral_shadow_value,
                                e,
                            )
                        })?;
                }
            }
        }

        // Clear the ephemeral states.
        self.ephemeral_balances.clear();
        self.ephemeral_shadow_allocs.clear();
        self.ephemeral_shadow_spaces.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_balances_backup.clear();
        self.ephemeral_shadow_allocs_backup.clear();
        self.ephemeral_shadow_spaces_backup.clear();

        Ok(())
    }
}
