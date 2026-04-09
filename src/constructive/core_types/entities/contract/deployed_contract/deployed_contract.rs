use crate::executive::executable::executable::Executable;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A struct for representing a deployed contract.
#[derive(Clone, Serialize, Deserialize)]
pub struct DeployedContract {
    // The id of the contract.
    pub contract_id: [u8; 32],

    // The executable of the contract.
    pub executable: Executable,

    // The registery index of the contract.
    pub registery_index: u64,
}

impl DeployedContract {
    /// Constructs a new deployed contract.
    pub fn new(contract_id: [u8; 32], executable: Executable, registery_index: u64) -> Self {
        Self {
            contract_id,
            executable,
            registery_index,
        }
    }

    /// Returns the deployed contract as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the JSON object.
        let mut obj = Map::new();

        // 2 Insert the contract id.
        obj.insert(
            "contract_id".to_string(),
            Value::String(hex::encode(self.contract_id)),
        );

        // 3 Insert the executable.
        obj.insert("executable".to_string(), self.executable.json());

        // 4 Is deployed? True.
        obj.insert("is_deployed".to_string(), Value::Bool(true));

        // 5 Insert the registery index.
        obj.insert(
            "registery_index".to_string(),
            Value::String(self.registery_index.to_string()),
        );

        // 6 Return the JSON object.
        Value::Object(obj)
    }
}

impl PartialEq for DeployedContract {
    fn eq(&self, other: &Self) -> bool {
        self.contract_id == other.contract_id
    }
}

impl Eq for DeployedContract {}
