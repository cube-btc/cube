use std::collections::HashMap;

/// State key.
type StateKey = Vec<u8>;

/// State value.
type StateValue = Vec<u8>;

/// A struct for holding contract states.
pub struct SMContractState {
    // Contract state.
    state: HashMap<StateKey, StateValue>,
}

impl SMContractState {
    /// Constructs a fresh new contract state.
    pub fn fresh_new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    /// Constructs a contract state from a hashmap.
    pub fn new(state: &HashMap<StateKey, StateValue>) -> Self {
        Self {
            state: state.clone(),
        }
    }

    /// Returns the state value by state key.
    pub fn get_state_value(&self, key: &StateKey) -> Option<StateValue> {
        self.state.get(key).cloned()
    }

    /// Inserts or updates a state by key.
    pub fn insert_update_state(&mut self, key: &StateKey, value: &StateValue) {
        self.state.insert(key.clone(), value.clone());
    }

    /// Removes a state by key.
    pub fn remove_state(&mut self, key: &StateKey) {
        self.state.remove(key);
    }
}
