use std::collections::HashMap;

/// Contract ID.
type ContractId = [u8; 32];

/// A variable size state key.
type StateKey = Vec<u8>;

/// A variable size state value.
type StateValue = Vec<u8>;

/// A struct for containing epheremal state differences to be applied for 'StateManager'.
#[derive(Clone)]
pub struct SMDelta {
    // New contracts to register.
    new_contracts_to_register: Vec<ContractId>,

    // New states to register for a given contract.
    new_contract_states: HashMap<ContractId, HashMap<StateKey, StateValue>>,

    // Updated states for a given contract.
    updated_contract_states: HashMap<ContractId, HashMap<StateKey, StateValue>>,

    // Removed states for a given contract.
    removed_contract_states: HashMap<ContractId, Vec<StateKey>>,
}

impl SMDelta {
    /// Constructs a fresh new state delta.
    pub fn new() -> Self {
        Self {
            new_contracts_to_register: Vec::new(),
            new_contract_states: HashMap::new(),
            updated_contract_states: HashMap::new(),
            removed_contract_states: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_contracts_to_register.clear();
        self.new_contract_states.clear();
        self.updated_contract_states.clear();
        self.removed_contract_states.clear();
    }

    /// Checks if a contract has just been epheremally registered in the delta.
    pub fn is_contract_epheremally_registered(&self, contract_id: ContractId) -> bool {
        self.new_contracts_to_register.contains(&contract_id)
    }

    /// Returns the value of a state by contract ID and key.
    pub fn get_epheremal_state_value(
        &self,
        contract_id: ContractId,
        key: &StateKey,
    ) -> Option<StateValue> {
        // 1 Return None if the state key has just been epheremally removed.
        if let Some(removed_states) = self.removed_contract_states.get(&contract_id) {
            if removed_states.contains(key) {
                return None;
            }
        }

        // 2 Try to get from the updated states first.
        if let Some(updated_states) = self.updated_contract_states.get(&contract_id) {
            if updated_states.contains_key(key) {
                return updated_states.get(key).cloned();
            }
        }

        // 3 Try to get from the new states.
        if let Some(new_states) = self.new_contract_states.get(&contract_id) {
            if new_states.contains_key(key) {
                return new_states.get(key).cloned();
            }
        }

        // 4 Return None if the state key is not found.
        None
    }

    /// Epheremally registers a contract into the delta.
    pub fn epheremally_register_contract(&mut self, contract_id: ContractId) {
        self.new_contracts_to_register.push(contract_id);
    }

    /// Epheremally inserts a new contract state.
    pub fn epheremally_insert_new_contract_state(
        &mut self,
        contract_id: ContractId,
        key: &StateKey,
        value: &StateValue,
    ) {
        // 1 Check if this key was epheremally removed.
        if let Some(removed_states) = self.removed_contract_states.get_mut(&contract_id) {
            // 1.1 If the key was epheremally removed, redo the removal.
            if removed_states.contains(key) {
                removed_states.retain(|k| k != key);
            }
        }

        // 2 Insert the new state into the new states.
        self.new_contract_states
            .entry(contract_id)
            .or_insert_with(HashMap::new)
            .insert(key.clone(), value.clone());
    }

    /// Epheremally updates an existing contract state.
    pub fn epheremally_update_existing_contract_state(
        &mut self,
        contract_id: ContractId,
        key: &StateKey,
        value: &StateValue,
    ) {
        self.updated_contract_states
            .entry(contract_id)
            .or_insert_with(HashMap::new)
            .insert(key.clone(), value.clone());
    }

    /// Epheremally removes a contract state.   
    pub fn epheremally_remove_existing_contract_state(
        &mut self,
        contract_id: ContractId,
        key: &StateKey,
    ) {
        self.removed_contract_states
            .entry(contract_id)
            .or_insert_with(Vec::new)
            .push(key.clone());
    }
}
