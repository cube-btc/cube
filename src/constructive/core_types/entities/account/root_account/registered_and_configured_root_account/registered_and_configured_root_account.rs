use crate::transmutative::bls::bls_ser::{deserialize_bls_key, serialize_bls_key};
use serde::{Deserialize, Serialize};

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
}

impl PartialEq for RegisteredAndConfiguredRootAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key
    }
}

impl Eq for RegisteredAndConfiguredRootAccount {}
