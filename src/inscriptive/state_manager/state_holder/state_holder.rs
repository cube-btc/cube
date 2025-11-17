use std::collections::HashMap;

/// State key.
type StateKey = Vec<u8>;

/// State value.
type StateValue = Vec<u8>;

/// A struct for holding contract states.
pub struct SMStateHolder {
    // Contract state.
    state: HashMap<StateKey, StateValue>,
}

impl SMStateHolder {
    /// Constructs a fresh new state holder.
    pub fn fresh_new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    /// Constructs a state holder from a hashmap.
    pub fn new(state: HashMap<StateKey, StateValue>) -> Self {
        Self { state: state }
    }

    /// Returns the contract state.
    pub fn state(&self) -> &HashMap<StateKey, StateValue> {
        &self.state
    }

    /// Returns a mutable reference to the contract state.
    pub fn state_mut(&mut self) -> &mut HashMap<StateKey, StateValue> {
        &mut self.state
    }

    /// Inserts a state into the holder.
    pub fn insert_state(&mut self, key: StateKey, value: StateValue) {
        self.state.insert(key, value);
    }

    /// Removes a state from the holder.
    pub fn remove_state(&mut self, key: StateKey) {
        self.state.remove(&key);
    }

    /// Gets a state from the holder.
    pub fn get_state(&self, key: StateKey) -> Option<&StateValue> {
        self.state.get(&key)
    }
}
