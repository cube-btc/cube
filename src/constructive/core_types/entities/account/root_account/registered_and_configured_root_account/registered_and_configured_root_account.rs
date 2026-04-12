use crate::transmutative::bls::bls_ser::{deserialize_bls_key, serialize_bls_key};
use crate::transmutative::secp::schnorr::Bytes32;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredAndConfiguredRootAccount {
    /// The Schnorr public key of the account.
    pub account_key: [u8; 32],

    /// The registery index of the account.
    pub registery_index: u64,

    /// The BLS public key of the account.
    #[serde(
        serialize_with = "serialize_bls_key",
        deserialize_with = "deserialize_bls_key"
    )]
    pub bls_key: [u8; 48],
}

impl RegisteredAndConfiguredRootAccount {
    /// Constructs a new registered and configured root account.
    pub fn new(account_key: [u8; 32], registery_index: u64, bls_key: [u8; 48]) -> Self {
        Self {
            account_key,
            registery_index,
            bls_key,
        }
    }

    /// Validates the `RegisteredAndConfiguredRootAccount`'s Schnorr and BLS keys.
    pub fn validate_schnorr_and_bls_key(&self) -> bool {
        // 1 Verify that the account key is indeed a valid Schnorr public key.
        if self.account_key.to_even_point().is_none() {
            return false;
        }

        // 2 Verify that the BLS key is indeed a valid BLS public key: TODO.

        // 3 Return true.
        true
    }

    /// Returns the registered and configured root account as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "kind".to_string(),
            Value::String("registered_and_configured".to_string()),
        );

        obj.insert(
            "account_key".to_string(),
            Value::String(hex::encode(self.account_key)),
        );

        obj.insert(
            "registery_index".to_string(),
            Value::String(self.registery_index.to_string()),
        );

        obj.insert(
            "bls_key".to_string(),
            Value::String(hex::encode(self.bls_key)),
        );

        Value::Object(obj)
    }
}

impl PartialEq for RegisteredAndConfiguredRootAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key
    }
}

impl Eq for RegisteredAndConfiguredRootAccount {}
