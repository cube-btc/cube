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
type CONTRACT_COIN_BALANCE = u64;

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
    /// In-memory cache of states: CONTRACT_ID -> (CONTRACT_COIN_BALANCE, { SHADOW_ACCOUNT_KEY -> SHADOW_ACCOUNT_COIN_BALANCE })
    coins: HashMap<CONTRACT_ID, (CONTRACT_COIN_BALANCE, CONTRACT_SHADOW_SPACE)>,
    /// Sled DB with all contract balances in a single tree.
    balance_db: sled::Db,
    /// Sled DB with contract shadow trees.
    shadows_db: sled::Db,
    /// In-memory cache of ephemeral states.
    ephemeral_coins: HashMap<CONTRACT_ID, (CONTRACT_COIN_BALANCE, CONTRACT_SHADOW_SPACE)>,
    /// In-memory cache of ephemeral states backup.
    ephemeral_coins_backup: HashMap<CONTRACT_ID, (CONTRACT_COIN_BALANCE, CONTRACT_SHADOW_SPACE)>,
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

        // Open the shadows db.
        let shadows_path = format!("db/{}/coin/contract/shadows", chain.to_string());
        let shadows_db = sled::open(shadows_path)
            .map_err(ContractCoinHolderConstructionError::ShadowsDBOpenError)?;

        // Initialize the in-memory cache of contract coins.
        let mut coins =
            HashMap::<CONTRACT_ID, (CONTRACT_COIN_BALANCE, CONTRACT_SHADOW_SPACE)>::new();

        // Iterate over all contract trees in the shadows db.
        for tree_name in shadows_db.tree_names() {
            let contract_id: CONTRACT_ID = tree_name.as_ref().try_into().map_err(|_| {
                ContractCoinHolderConstructionError::InvalidContractIDBytes(tree_name.to_vec())
            })?;

            // Open the contract tree.
            let tree = shadows_db.open_tree(&tree_name).map_err(|e| {
                ContractCoinHolderConstructionError::ShadowsTreeOpenError(contract_id, e)
            })?;

            // Initialize the in-memory cache of contract shadows.
            let mut contract_shadows =
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

                // Insert the shadow into the in-memory cache.
                contract_shadows.insert(shadow_account_key, shadow_value);
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

            // Insert the contract shadows and balance into the in-memory cache.
            coins.insert(contract_id, (contract_balance, contract_shadows));
        }

        // Create the state holder.
        let contract_coin_holder = ContractCoinHolder {
            coins,
            balance_db,
            shadows_db,
            ephemeral_coins:
                HashMap::<CONTRACT_ID, (CONTRACT_COIN_BALANCE, CONTRACT_SHADOW_SPACE)>::new(),
            ephemeral_coins_backup: HashMap::<
                CONTRACT_ID,
                (CONTRACT_COIN_BALANCE, CONTRACT_SHADOW_SPACE),
            >::new(),
        };

        // Return the guarded state holder.
        Ok(Arc::new(Mutex::new(contract_coin_holder)))
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

    /// Get the contract coin balance for a contract ID.
    pub async fn get_contract_balance(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some((balance, _)) = self.ephemeral_coins.get(&contract_id) {
            return Some(*balance);
        }

        // And then try to get from the states.
        self.coins.get(&contract_id).map(|(balance, _)| *balance)
    }

    /// Get the shadow balance of an account for a specific contract ID.
    pub async fn get_shadow_balance_of_an_account_for_a_contract(
        &self,
        contract_id: [u8; 32],
        shadow_account_key: SHADOW_ACCOUNT_KEY,
    ) -> Option<u64> {
        // Try to get from the ephemeral states first.
        if let Some((_, shadow_balance)) = self.ephemeral_coins.get(&contract_id) {
            return shadow_balance.get(&shadow_account_key).cloned();
        }

        // And then try to get from the states.
        self.coins
            .get(&contract_id)
            .and_then(|(_, shadow_balance)| shadow_balance.get(&shadow_account_key).cloned())
    }

    /// Inserts or updates a shadow balance by key and contract ID ephemerally.
    pub async fn insert_update_shadow_balance(
        &mut self,
        contract_id: [u8; 32],
        shadow_account_key: SHADOW_ACCOUNT_KEY,
        shadow_value: SHADOW_ACCOUNT_COIN_BALANCE,
    ) -> Option<()> {
        // Get mutable ephemeral states.
        let ephemeral_coins = match self.ephemeral_coins.get_mut(&contract_id) {
            Some((balance, shadows)) => (balance, shadows),
            None => {
                // Create it if it doesn't exist.
                let contract_shadows =
                    HashMap::<SHADOW_ACCOUNT_KEY, SHADOW_ACCOUNT_COIN_BALANCE>::new();

                // Insert it.
                self.ephemeral_coins
                    .insert(contract_id, (0, contract_shadows));

                // Get it again.
                let (balance, shadows) = self.ephemeral_coins.get_mut(&contract_id).unwrap(); // Safe because we just inserted it.
                (balance, shadows)
            }
        };

        // Insert (or update) the value into the ephemeral states.
        ephemeral_coins.1.insert(shadow_account_key, shadow_value);

        // Return the result.
        Some(())
    }

    /// Updates the contract coin balance for a contract ID ephemerally.
    pub async fn update_contract_balance(
        &mut self,
        contract_id: [u8; 32],
        new_balance: CONTRACT_COIN_BALANCE,
    ) -> Option<()> {
        // Get mutable ephemeral states.
        let ephemeral_coins = match self.ephemeral_coins.get_mut(&contract_id) {
            Some((balance, shadows)) => (balance, shadows),
            None => {
                // Create it if it doesn't exist.
                let contract_shadows =
                    HashMap::<SHADOW_ACCOUNT_KEY, SHADOW_ACCOUNT_COIN_BALANCE>::new();

                // Insert it.
                self.ephemeral_coins
                    .insert(contract_id, (0, contract_shadows));

                // Get it again.
                let (balance, shadows) = self.ephemeral_coins.get_mut(&contract_id).unwrap(); // Safe because we just inserted it.
                (balance, shadows)
            }
        };

        // Update the balance in the ephemeral states.
        *ephemeral_coins.0 = new_balance;

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
        self.ephemeral_coins.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_coins_backup.clear();
    }

    /// Saves the states updated associated with all executions (on-disk and in-memory).
    ///
    /// TODO Performance Optimization: Open the tree *once per contract ID* and then insert all key-values at once.
    pub fn save_all_executions(&mut self) -> Result<(), ContractCoinHolderSaveError> {
        // Iterate over all ephemeral states.
        for (contract_id, (ephemeral_balance, ephemeral_shadows)) in self.ephemeral_coins.iter() {
            // In-memory insertion.
            {
                // Get mutable states.
                let coins = match self.coins.get_mut(contract_id) {
                    Some(coins) => coins,
                    None => {
                        // Create it if it doesn't exist.
                        let contract_shadows =
                            HashMap::<SHADOW_ACCOUNT_KEY, SHADOW_ACCOUNT_COIN_BALANCE>::new();

                        // Insert it.
                        self.coins
                            .insert(contract_id.to_owned(), (0, contract_shadows));

                        // Get it again.
                        self.coins.get_mut(contract_id).unwrap() // Safe because we just inserted it.
                    }
                };

                // Update the balance and allocations in memory.
                coins.0 = *ephemeral_balance;

                // Iterate over all ephemeral allocations for this contract.
                for (ephemeral_shadow_account_key, ephemeral_shadow_value) in
                    ephemeral_shadows.iter()
                {
                    coins.1.insert(
                        ephemeral_shadow_account_key.clone(),
                        *ephemeral_shadow_value,
                    );
                }
            }

            // On-disk insertion.
            {
                // Save the balance to the balances db.
                self.balance_db
                    .insert(&contract_id, ephemeral_balance.to_le_bytes().to_vec())
                    .map_err(|e| {
                        ContractCoinHolderSaveError::TreeValueInsertError(
                            contract_id.to_owned(),
                            contract_id.to_vec(),
                            *ephemeral_balance,
                            e,
                        )
                    })?;

                // Save the shadows to the shadows db.
                let shadows_tree = self.shadows_db.open_tree(&contract_id).map_err(|e| {
                    ContractCoinHolderSaveError::OpenTreeError(contract_id.to_owned(), e)
                })?;

                // Insert all shadows into the on-disk contract tree.
                for (ephemeral_shadow_account_key, ephemeral_shadow_value) in
                    ephemeral_shadows.iter()
                {
                    shadows_tree
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
        self.ephemeral_coins.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_coins_backup.clear();

        Ok(())
    }
}
