use super::contract_state::contract_state::SMContractState;
use super::delta::delta::SMDelta;
use super::errors::construction_error::SMConstructionError;
use super::errors::insert_update_state_error::SMInsertUpdateStateError;
use super::errors::register_error::SMRegisterContractError;
use crate::inscriptive::state_manager::errors::remove_state_error::SMRemoveStateError;
use crate::operative::Chain;
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
    // In-memory contract states.
    pub in_memory_states: HashMap<ContractId, SMContractState>,

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
        // 1 Open the state db.
        let state_db_path = format!("db/{}/state", chain.to_string());
        let state_db = sled::open(state_db_path).map_err(SMConstructionError::DBOpenError)?;

        // 2 Initialize the in-memory states.
        let mut in_memory_states = HashMap::<ContractId, SMContractState>::new();

        // 3 Collect states from the state db.
        for tree_name in state_db.tree_names() {
            // 3.1 Deserialize contract id bytes from tree name.
            let contract_id: [u8; 32] = match tree_name.as_ref().try_into() {
                Ok(key) => key,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };

            // 3.2 Open the tree.
            let tree = state_db
                .open_tree(tree_name)
                .map_err(|e| SMConstructionError::TreeOpenError(contract_id, e))?;

            // 3.3 Collect the contract state from the tree.
            let contract_state: HashMap<StateKey, StateValue> = tree
                .iter()
                .filter_map(|res| res.ok())
                .map(|(k, v)| (k.to_vec(), v.to_vec()))
                .collect::<HashMap<StateKey, StateValue>>();

            // 3.4 Construct the contract state from the collected values.
            let contract_state = SMContractState::new(contract_state);

            // 3.5 Insert the contract state into the in-memory states.
            in_memory_states.insert(contract_id, contract_state);
        }

        // 4 Construct the state manager.
        let state_manager = StateManager {
            in_memory_states,
            on_disk_states: state_db,
            delta: SMDelta::new(),
            backup_of_delta: SMDelta::new(),
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
    #[allow(unused)]
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
        // 1 Try to get from the delta first.
        if let Some(value) = self.delta.get_epheremal_state_value(contract_id, key) {
            return Some(value.clone());
        }

        // 2 And then try to get from the permanent in-memory states.
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
                // 2.a.1 Insert the new value to the delta.
                self.delta
                    .epheremally_update_existing_contract_state(contract_id, key, value);

                // 2.a.2 Return the previous value for updated.
                return Ok(Some(existing_value));
            }
            // 2.b Insert the value.
            None => {
                // 2.b.1 Insert the value.
                self.delta
                    .epheremally_insert_new_contract_state(contract_id, key, value);

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
}
