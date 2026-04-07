use crate::executive::executable::executable::Executable;
use serde_json::{Map, Value};

/// A struct for containing the registery index and call counter of a contract.
#[derive(Clone)]
pub struct RMContractBody {
    // Assigned registery index of a deployed contract.
    pub registery_index: u64,

    // Ever-increasing call counter of a contract.
    pub call_counter: u64,

    // Decompiled executable of a contract.
    pub executable: Executable,
}

impl RMContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(registery_index: u64, call_counter: u64, executable: Executable) -> Self {
        Self {
            registery_index,
            call_counter,
            executable,
        }
    }

    /// Returns the contract body as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the contract body JSON object.
        let mut obj = Map::new();

        // 2 Insert the registery index.
        obj.insert(
            "registery_index".to_string(),
            Value::String(self.registery_index.to_string()),
        );

        // 3 Insert the call counter.
        obj.insert(
            "call_counter".to_string(),
            Value::String(self.call_counter.to_string()),
        );

        // 4 Insert the executable.
        obj.insert("executable".to_string(), self.executable.json());

        // 5 Return the contract body JSON object.
        Value::Object(obj)
    }
}
