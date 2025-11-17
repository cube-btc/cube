use std::collections::HashMap;

/// Contract ID.
type ContractId = [u8; 32];

/// A variable size state key.
type StateKey = Vec<u8>;

/// A variable size state value.
type StateValue = Vec<u8>;

/// A struct for containing state differences to be applied for 'StateManager'.
#[derive(Clone)]
pub struct SMDelta {
    // New contracts to register.
    new_contracts_to_register: Vec<ContractId>,

    // New states to register for a given contract.
    new_contract_states: HashMap<ContractId, HashMap<StateKey, StateValue>>,

    // Updated contract states for a given contract.
    updated_contract_states: HashMap<ContractId, HashMap<StateKey, StateValue>>,

    // Removed states for a given contract.
    removed_contract_states: HashMap<ContractId, HashMap<StateKey, StateValue>>,
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
}
