use super::state_holder_error::{StateHolderConstructionError, StateHolderSaveError};
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Contract ID: 32-byte unique identifier.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// State key.
#[allow(non_camel_case_types)]
type STATE_KEY = Vec<u8>;

/// State value.
#[allow(non_camel_case_types)]
type STATE_VALUE = Vec<u8>;

/// A struct for containing contract/program states in-memory and on-disk.
pub struct StateHolder {
    /// In-memory cache of states: CONTRACT_ID -> { STATE_KEY -> STATE_VALUE }
    states: HashMap<CONTRACT_ID, HashMap<STATE_KEY, STATE_VALUE>>,
    /// Sled DB with contract trees.
    states_db: sled::Db,
    /// In-memory cache of ephemeral states.
    ephemeral_states: HashMap<CONTRACT_ID, HashMap<STATE_KEY, STATE_VALUE>>,
    /// In-memory cache of ephemeral states backup.
    ephemeral_states_backup: HashMap<CONTRACT_ID, HashMap<STATE_KEY, STATE_VALUE>>,
}

/// Guarded state holder.
#[allow(non_camel_case_types)]
pub type STATE_HOLDER = Arc<Mutex<StateHolder>>;

// TODO: Implement a rank-based caching mechanism to only cache the high-ranked states.
// Right now, we are caching *ALL* contract states in memory.
impl StateHolder {
    /// Initialize the state for the given chain
    pub fn new(chain: Chain) -> Result<STATE_HOLDER, StateHolderConstructionError> {
        // Open the main state db.
        let path = format!("db/{}/state", chain.to_string());
        let states_db = sled::open(path).map_err(StateHolderConstructionError::MainDBOpenError)?;

        // Initialize the in-memory cache of contract states.
        let mut states = HashMap::<CONTRACT_ID, HashMap<STATE_KEY, STATE_VALUE>>::new();

        // Iterate over all contract trees in the main state db.
        for tree_name in states_db.tree_names() {
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                StateHolderConstructionError::InvalidContractIDBytes(tree_name.to_vec())
            })?;

            // Open the contract tree.
            let tree = states_db
                .open_tree(&tree_name)
                .map_err(|e| StateHolderConstructionError::SubDBOpenError(contract_id, e))?;

            // Iterate over all items in the contract tree.
            let contract_state = tree
                .iter()
                .filter_map(|res| res.ok())
                .map(|(k, v)| (k.to_vec(), v.to_vec()))
                .collect::<HashMap<STATE_KEY, STATE_VALUE>>();

            // Insert the contract state into the in-memory cache.
            states.insert(contract_id, contract_state);
        }

        // Create the state holder.
        let state_holder = StateHolder {
            states,
            states_db,
            ephemeral_states: HashMap::<CONTRACT_ID, HashMap<STATE_KEY, STATE_VALUE>>::new(),
            ephemeral_states_backup: HashMap::<CONTRACT_ID, HashMap<STATE_KEY, STATE_VALUE>>::new(),
        };

        // Return the guarded state holder.
        Ok(Arc::new(Mutex::new(state_holder)))
    }

    /// Clones ephemeral states into the backup.
    fn backup_ephemeral_states(&mut self) {
        self.ephemeral_states_backup = self.ephemeral_states.clone();
    }

    /// Restores ephemeral states from the backup.
    fn restore_ephemeral_states(&mut self) {
        self.ephemeral_states = self.ephemeral_states_backup.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the ephemeral states.
        self.backup_ephemeral_states();
    }

    /// Get the value by key and contract ID.
    pub fn get_value(&self, key: &STATE_KEY, contract_id: &CONTRACT_ID) -> Option<STATE_VALUE> {
        // Try to get from the ephemeral states first.
        if let Some(state) = self.ephemeral_states.get(contract_id) {
            return state.get(key).cloned();
        }

        // And then try to get from the states.
        self.states
            .get(contract_id)
            .and_then(|state| state.get(key).cloned())
    }

    /// Inserts or updates a value by key and contract ID ephemerally.
    pub fn insert_value(
        &mut self,
        contract_id: &CONTRACT_ID,
        key: &STATE_KEY,
        value: &STATE_VALUE,
    ) {
        // Get mutable ephemeral states.
        let ephemeral_contract_states = match self.ephemeral_states.get_mut(contract_id) {
            Some(states) => states,
            None => {
                // Create it if it doesn't exist.
                let contract_states = HashMap::<STATE_KEY, STATE_VALUE>::new();

                // Insert it.
                self.ephemeral_states.insert(*contract_id, contract_states);

                // Get it again.
                self.ephemeral_states.get_mut(contract_id).unwrap() // Safe because we just inserted it.
            }
        };

        // Insert (or update) the value into the ephemeral states.
        ephemeral_contract_states.insert(key.clone(), value.clone());
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
        self.ephemeral_states.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_states_backup.clear();
    }

    /// Saves the states updated associated with all executions (on-disk and in-memory).
    ///
    /// TODO Performance Optimization: Open the tree *once per contract ID* and then insert all key-values at once.
    pub fn save_all_executions(&mut self) -> Result<(), StateHolderSaveError> {
        // Iterate over all ephemeral states.
        for (contract_id, ephemeral_contract_states) in self.ephemeral_states.iter() {
            // Iterate over all items in the contract state.
            for (ephemeral_state_key, ephemeral_state_value) in ephemeral_contract_states.iter() {
                // In-memory insertion.
                {
                    // Get mutable states.
                    let states = match self.states.get_mut(contract_id) {
                        Some(states) => states,
                        None => {
                            // Create it if it doesn't exist.
                            let contract_states = HashMap::<STATE_KEY, STATE_VALUE>::new();

                            // Insert it.
                            self.states.insert(*contract_id, contract_states);

                            // Get it again.
                            self.states.get_mut(contract_id).unwrap() // Safe because we just inserted it.
                        }
                    };

                    // Insert the value into the in-memory contract states.
                    states.insert(ephemeral_state_key.clone(), ephemeral_state_value.clone());
                }

                // On-disk insertion.
                {
                    // Open the contract tree.
                    let tree = self
                        .states_db
                        .open_tree(contract_id)
                        .map_err(|e| StateHolderSaveError::OpenTreeError(*contract_id, e))?;

                    // Insert the value into the on-disk contract tree.
                    tree.insert(ephemeral_state_key, ephemeral_state_value.clone())
                        .map_err(|e| {
                            StateHolderSaveError::TreeValueInsertError(
                                *contract_id,
                                ephemeral_state_key.clone(),
                                ephemeral_state_value.clone(),
                                e,
                            )
                        })?;
                }
            }
        }

        // Clear the ephemeral states.
        self.ephemeral_states.clear();

        // Clear the ephemeral states backup.
        self.ephemeral_states_backup.clear();

        Ok(())
    }
}
