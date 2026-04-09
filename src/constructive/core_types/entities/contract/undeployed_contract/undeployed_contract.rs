use crate::executive::executable::executable::Executable;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A struct for representing an undeployed contract.
#[derive(Clone, Serialize, Deserialize)]
pub struct UndeployedContract {
    // The id of the contract.
    pub contract_id: [u8; 32],

    // The executable of the contract.
    pub executable: Executable,
}

impl UndeployedContract {
    /// Constructs a new undeployed contract.
    pub fn new(contract_id: [u8; 32], executable: Executable) -> Self {
        Self {
            contract_id,
            executable,
        }
    }

    /// Returns the undeployed contract as a JSON object.
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

        // 4 Is deployed? False.
        obj.insert("is_deployed".to_string(), Value::Bool(false));

        // 5 Return the JSON object.
        Value::Object(obj)
    }
}

impl PartialEq for UndeployedContract {
    fn eq(&self, other: &Self) -> bool {
        self.contract_id == other.contract_id
    }
}

impl Eq for UndeployedContract {}
