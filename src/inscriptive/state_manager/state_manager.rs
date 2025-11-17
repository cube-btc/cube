use super::contract_state::contract_state::SMContractState;
use super::delta::delta::SMDelta;
use super::errors::construction_errors::SMConstructionError;
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
}
