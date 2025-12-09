use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A struct for representing an unregistered account.
#[derive(Clone, Serialize, Deserialize)]
pub struct UnregisteredAccount {
    /// The secp256k1 public key of the account.
    pub key: [u8; 32],
}

impl UnregisteredAccount {
    /// Constructs a new unregistered account.
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Returns the unregistered account as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the unregistered account JSON object.
        let mut obj = Map::new();

        // 2 Insert the key.
        obj.insert(
            "account_key".to_string(),
            Value::String(hex::encode(self.key)),
        );

        // 3 Is registered? False.
        obj.insert("is_registered".to_string(), Value::Bool(false));

        // 4 Return the JSON object.
        Value::Object(obj)
    }
}

impl PartialEq for UnregisteredAccount {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for UnregisteredAccount {}
