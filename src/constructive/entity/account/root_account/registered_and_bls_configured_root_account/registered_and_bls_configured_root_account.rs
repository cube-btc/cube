use crate::constructive::ser::{deserialize_bls_key, serialize_bls_key};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredAndBLSConfiguredRootAccount {
    /// The secp256k1 public key of the account.
    pub account_key: [u8; 32],

    /// The BLS key of the account.
    #[serde(
        serialize_with = "serialize_bls_key",
        deserialize_with = "deserialize_bls_key"
    )]
    pub bls_key: [u8; 48],
}

impl RegisteredAndBLSConfiguredRootAccount {
    pub fn new(account_key: [u8; 32], bls_key: [u8; 48]) -> Self {
        Self {
            account_key,
            bls_key,
        }
    }
}
