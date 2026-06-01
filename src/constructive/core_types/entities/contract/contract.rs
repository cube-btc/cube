use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A deployed contract registered with the registry.
#[derive(Clone, Serialize, Deserialize)]
pub struct Contract {
    pub contract_id: [u8; 32],
    pub registry_index: u64,
}

impl Contract {
    pub fn new(contract_id: [u8; 32], registry_index: u64) -> Self {
        Self {
            contract_id,
            registry_index,
        }
    }

    pub fn contract_id(&self) -> [u8; 32] {
        self.contract_id
    }

    pub fn registry_index(&self) -> u64 {
        self.registry_index
    }

    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "contract_id".to_string(),
            Value::String(hex::encode(self.contract_id)),
        );

        obj.insert(
            "registry_index".to_string(),
            Value::String(self.registry_index.to_string()),
        );

        Value::Object(obj)
    }
}

impl PartialEq for Contract {
    fn eq(&self, other: &Self) -> bool {
        self.contract_id == other.contract_id
    }
}

impl Eq for Contract {}
