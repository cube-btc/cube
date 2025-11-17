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
    /// Constructs a contract state from a hashmap.
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
}
