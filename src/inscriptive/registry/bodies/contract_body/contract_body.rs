use crate::executive::executable::executable::Executable;
use serde_json::{Map, Value};

/// A struct for containing the registry index and call counter of a contract.
#[derive(Clone)]
pub struct RMContractBody {
    // Assigned registry index of a deployed contract.
    pub registry_index: u64,

    // Ever-increasing call counter of a contract.
    pub call_counter: u64,

    // Last observed activity timestamp of a contract.
    pub last_activity_timestamp: u64,

    // Decompiled executable of a contract.
    pub executable: Executable,
}

impl RMContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(
        registry_index: u64,
        call_counter: u64,
        last_activity_timestamp: u64,
        executable: Executable,
    ) -> Self {
        Self {
            registry_index,
            call_counter,
            last_activity_timestamp,
            executable,
        }
    }

    /// Returns the contract body as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the contract body JSON object.
        let mut obj = Map::new();

        // 2 Insert the registry index.
        obj.insert(
            "registry_index".to_string(),
            Value::String(self.registry_index.to_string()),
        );

        // 3 Insert the call counter.
        obj.insert(
            "call_counter".to_string(),
            Value::String(self.call_counter.to_string()),
        );

        // 4 Insert the last activity timestamp.
        obj.insert(
            "last_activity_timestamp".to_string(),
            Value::String(self.last_activity_timestamp.to_string()),
        );

        // 5 Insert the executable.
        obj.insert("executable".to_string(), self.executable.json());

        // 6 Return the contract body JSON object.
        Value::Object(obj)
    }
}
