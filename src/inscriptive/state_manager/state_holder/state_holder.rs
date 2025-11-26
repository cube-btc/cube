use serde_json::{Map, Value};
use std::collections::HashMap;

/// State key.
type StateKey = Vec<u8>;

/// State value.
type StateValue = Vec<u8>;

/// A struct for holding contract states.
pub struct SMContractStateHolder {
    // Contract states.
    states: HashMap<StateKey, StateValue>,
}

impl SMContractStateHolder {
    /// Constructs a fresh new state holder.
    pub fn fresh_new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Constructs a state holder from a hashmap.
    pub fn new(states: &HashMap<StateKey, StateValue>) -> Self {
        Self {
            states: states.clone(),
        }
    }

    /// Returns the state value by state key.
    pub fn get_state_value(&self, key: &StateKey) -> Option<StateValue> {
        self.states.get(key).cloned()
    }

    /// Inserts or updates a state by key.
    pub fn insert_update_state(&mut self, key: &StateKey, value: &StateValue) {
        self.states.insert(key.clone(), value.clone());
    }

    /// Removes a state by key.
    pub fn remove_state(&mut self, key: &StateKey) {
        self.states.remove(key);
    }

    /// Returns the state holder as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the state holder JSON object.
        let mut obj = Map::new();

        // 2 Insert the states. I think we can direct insert key and values as strings.
        obj.insert(
            "contract_states".to_string(),
            Value::Object(
                self.states
                    .iter()
                    .map(|(key, value)| (hex::encode(key), Value::String(hex::encode(value))))
                    .collect(),
            ),
        );

        // 3 Return the JSON object.
        Value::Object(obj)
    }
}
