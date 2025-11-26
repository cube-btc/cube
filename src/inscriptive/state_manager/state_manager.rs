use super::delta::delta::SMDelta;
use super::errors::construction_error::SMConstructionError;
use super::errors::insert_update_state_error::SMInsertUpdateStateError;
use super::errors::register_error::SMRegisterContractError;
use crate::inscriptive::state_manager::errors::apply_changes_error::SMApplyChangesError;
use crate::inscriptive::state_manager::errors::remove_state_error::SMRemoveStateError;
use crate::inscriptive::state_manager::state_holder::state_holder::SMContractStateHolder;
use crate::operative::Chain;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Contract ID.
type ContractId = [u8; 32];

/// State key.
type StateKey = Vec<u8>;

/// State value.
type StateValue = Vec<u8>;

/// A struct for managing contract states in-memory and on-disk.
pub struct StateManager {
    // In-memory states.
    pub in_memory_states: HashMap<ContractId, SMContractStateHolder>,

    // On-disk states.
    pub on_disk_states: sled::Db,

    // State differences to be applied.
    pub delta: SMDelta,

    // Backup of state differences in case of rollback.
    pub backup_of_delta: SMDelta,
}

// Guarded 'StateManager'.
#[allow(non_camel_case_types)]
pub type STATE_MANAGER = Arc<Mutex<StateManager>>;

impl StateManager {
    /// Constructs a fresh new 'StateManager'.
    pub fn new(chain: Chain) -> Result<STATE_MANAGER, SMConstructionError> {
        // 1 Open the states db.
        let states_db_path = format!("storage/{}/states", chain.to_string());
        let states_db = sled::open(states_db_path).map_err(SMConstructionError::DBOpenError)?;

        // 2 Initialize the in-memory states.
        let mut in_memory_states = HashMap::<ContractId, SMContractStateHolder>::new();

        // 3 Collect states from the database.
        for tree_name in states_db.tree_names() {
            // 3.1 Deserialize contract id bytes from tree name.
            let contract_id: [u8; 32] = match tree_name.as_ref().try_into() {
                Ok(key) => key,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };

            // 3.2 Open the tree.
            let tree = states_db
                .open_tree(tree_name)
                .map_err(|e| SMConstructionError::TreeOpenError(contract_id, e))?;

            // 3.3 Collect the contract states from the tree.
            let states: HashMap<StateKey, StateValue> = tree
                .iter()
                .filter_map(|res| res.ok())
                .map(|(k, v)| (k.to_vec(), v.to_vec()))
                .collect::<HashMap<StateKey, StateValue>>();

            // 3.4 Construct the state holder from the collected values.
            let state_holder = SMContractStateHolder::new(&states);

            // 3.5 Insert the state holder into the in-memory states.
            in_memory_states.insert(contract_id, state_holder);
        }

        // 4 Construct the state manager.
        let state_manager = StateManager {
            in_memory_states,
            on_disk_states: states_db,
            delta: SMDelta::fresh_new(),
            backup_of_delta: SMDelta::fresh_new(),
        };

        // 5 Guard the state manager.
        let guarded_state_manager = Arc::new(Mutex::new(state_manager));

        // 6 Return the guarded state manager.
        Ok(guarded_state_manager)
    }

    /// Clones the delta into the backup.
    fn backup_delta(&mut self) {
        self.backup_of_delta = self.delta.clone();
    }

    /// Restores the delta from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.backup_of_delta.clone();
    }

    /// Prepares the state manager prior to each execution.
    ///
    /// NOTE: Used by the Engine.
    pub fn pre_execution(&mut self) {
        // Backup the delta.
        self.backup_delta();
    }

    /// Checks if a contract is permanently registered.
    pub fn is_contract_registered(&self, contract_id: ContractId) -> bool {
        self.in_memory_states.contains_key(&contract_id)
    }

    /// Returns the value of a state by contract ID and key.
    pub fn get_state_value(&self, contract_id: ContractId, key: &StateKey) -> Option<StateValue> {
        // 1 Check if the state has just been epheremally removed in the delta.
        if self.delta.is_state_epheremally_removed(contract_id, key) {
            return None;
        }

        // 2 Try to get from the delta first.
        if let Some(value) = self.delta.get_epheremal_state_value(contract_id, key) {
            return Some(value.clone());
        }

        // 3 And then try to get from the permanent in-memory states.
        self.in_memory_states
            .get(&contract_id)?
            .get_state_value(key)
    }

    /// Registers a new contract.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_contract(
        &mut self,
        contract_id: ContractId,
    ) -> Result<(), SMRegisterContractError> {
        // 1 Check if the contract has just ben epheremally registered in the delta.
        if self.delta.is_contract_epheremally_registered(contract_id) {
            return Err(
                SMRegisterContractError::ContractHasJustBeenEphemerallyRegistered(contract_id),
            );
        }

        // 2 Check if the contract is already permanently registered.
        if self.is_contract_registered(contract_id) {
            return Err(
                SMRegisterContractError::ContractIsAlreadyPermanentlyRegistered(contract_id),
            );
        }

        // 3 Epheremally register the contract in the delta.
        self.delta.epheremally_register_contract(contract_id);

        // 4 Return the result.
        Ok(())
    }

    /// Inserts or updates a value by key and contract ID into the ephemeral states.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn insert_update_state(
        &mut self,
        contract_id: ContractId,
        key: &StateKey,
        value: &StateValue,
        optimized: bool,
    ) -> Result<Option<StateValue>, SMInsertUpdateStateError> {
        // 1 If not optimized, check if the contract is registered.
        if !optimized {
            if !self.is_contract_registered(contract_id) {
                return Err(SMInsertUpdateStateError::ContractNotRegistered(contract_id));
            }
        }

        // 2 Check if the value already exists.
        match self.get_state_value(contract_id, key) {
            // 2.a Update the existing value.
            Some(existing_value) => {
                // 2.a.1 Epheremally insert the updated value to the delta.
                self.delta.epheremally_insert_new_or_updated_contract_state(
                    contract_id,
                    key,
                    value,
                );

                // 2.a.2 Return the previous value for updated.
                return Ok(Some(existing_value));
            }
            // 2.b Insert the value.
            None => {
                // 2.b.1 Epheremally insert the new value to the delta.
                self.delta.epheremally_insert_new_or_updated_contract_state(
                    contract_id,
                    key,
                    value,
                );

                // 2.b.2 Return None for newly inserted.
                return Ok(None);
            }
        }
    }

    /// Removes a state by key and contract ID from the ephemeral states.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn remove_state(
        &mut self,
        contract_id: ContractId,
        key: &StateKey,
        optimized: bool,
    ) -> Result<(), SMRemoveStateError> {
        // 1 If not optimized, check if the contract is registered.
        if !optimized {
            if !self.is_contract_registered(contract_id) {
                return Err(SMRemoveStateError::ContractNotRegistered(contract_id));
            }
        }

        // 2 Return error if the state does not exist.
        if let None = self.get_state_value(contract_id, key) {
            return Err(SMRemoveStateError::StateDoesNotExist(
                contract_id,
                key.clone(),
            ));
        }

        // 3 Epheremally remove the state in the delta.
        self.delta
            .epheremally_remove_existing_contract_state(contract_id, key);

        // 4 Return the result.
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
        self.delta.flush();

        // Clear the ephemeral states backup.
        self.backup_of_delta.flush();
    }

    /// Applies the changes to the 'StateManager'.
    pub fn apply_changes(&mut self) -> Result<(), SMApplyChangesError> {
        // 1 Apply the new contracts to register.
        for contract_id in self.delta.new_contracts_to_register.iter() {
            // 1.1 On-disk insertion.
            {
                // 1.1.1 Open the tree. This creates a new tree since it does not exist.
                self.on_disk_states
                    .open_tree(contract_id)
                    .map_err(|e| SMApplyChangesError::TreeOpenError(contract_id.clone(), e))?;
            }

            // 1.2 In-memory insertion.
            {
                // 1.2.1 Create a fresh new contract state holder.
                let fresh_new_contract_state_holder = SMContractStateHolder::fresh_new();

                // 1.2.2 Insert the contract state into the in-memory states.
                self.in_memory_states
                    .insert(contract_id.clone(), fresh_new_contract_state_holder);
            }
        }

        // 2 Apply the new or updated states.
        for (contract_id, epheremal_states) in self.delta.new_or_updated_contract_states.iter() {
            // 2.1 On-disk insertion FIRST (critical: apply on-disk before in-memory for atomicity).
            {
                // 2.1.1 Open the tree.
                let tree = self
                    .on_disk_states
                    .open_tree(contract_id)
                    .map_err(|e| SMApplyChangesError::TreeOpenError(contract_id.clone(), e))?;

                // 2.1.2 Insert the states into the tree.
                for (epheremal_state_key, epheremal_state_value) in epheremal_states.iter() {
                    tree.insert(epheremal_state_key, epheremal_state_value.clone())
                        .map_err(|e| {
                            SMApplyChangesError::TreeValueInsertError(
                                contract_id.clone(),
                                epheremal_state_key.clone(),
                                epheremal_state_value.clone(),
                                e,
                            )
                        })?;
                }
            }

            // 2.2 In-memory insertion.
            {
                // 2.2.1 Get the mutable contract state holder from the in-memory states.
                let mut_contract_state_holder = self.in_memory_states.get_mut(contract_id).ok_or(
                    SMApplyChangesError::ContractIdNotFoundInMemory(contract_id.clone()),
                )?;

                // 2.2.2 Insert the states into the contract state holder.
                for (epheremal_state_key, epheremal_state_value) in epheremal_states.iter() {
                    mut_contract_state_holder
                        .insert_update_state(epheremal_state_key, epheremal_state_value);
                }
            }
        }

        // 3 Apply the removed states.
        for (contract_id, state_keys_to_remove) in self.delta.removed_contract_states.iter() {
            // 3.1 On-disk removal.
            {
                // 3.1.1 Open the tree.
                let tree = self
                    .on_disk_states
                    .open_tree(contract_id)
                    .map_err(|e| SMApplyChangesError::TreeOpenError(contract_id.clone(), e))?;

                // 3.1.2 Remove the states from the tree.
                for state_key_to_remove in state_keys_to_remove.iter() {
                    tree.remove(state_key_to_remove).map_err(|e| {
                        SMApplyChangesError::TreeValueRemoveError(
                            contract_id.clone(),
                            state_key_to_remove.clone(),
                            e,
                        )
                    })?;
                }
            }

            // 3.2 In-memory removal.
            {
                // 3.2.1 Get the mutable contract state holder from the in-memory states.
                let mut_contract_state_holder = self.in_memory_states.get_mut(contract_id).ok_or(
                    SMApplyChangesError::ContractIdNotFoundInMemory(*contract_id),
                )?;

                // 3.2.2 Remove the states from the contract state holder.
                for state_key_to_remove in state_keys_to_remove.iter() {
                    mut_contract_state_holder.remove_state(state_key_to_remove);
                }
            }
        }

        // 4 Return the result.
        Ok(())
    }

    /// Returns the state manager as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the state manager JSON object.
        let mut obj = Map::new();

        // 2 Insert the contract states.
        obj.insert(
            "contracts".to_string(),
            Value::Object(
                self.in_memory_states
                    .iter()
                    .map(|(contract_id, contract_state_holder)| {
                        (hex::encode(contract_id), contract_state_holder.json())
                    })
                    .collect(),
            ),
        );

        // 3 Return the JSON object.
        Value::Object(obj)
    }
}

/// Erases the state manager by db path.
pub fn erase_state_manager(chain: Chain) {
    // States db path.
    let states_db_path = format!("storage/{}/states", chain.to_string());

    // Erase the path.
    let _ = std::fs::remove_dir_all(states_db_path);
}
