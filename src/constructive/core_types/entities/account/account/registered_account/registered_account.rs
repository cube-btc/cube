use serde::{Deserialize, Serialize};

/// A registered account.
#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredAccount {
    /// The Schnorr public key of the account.
    pub account_key: [u8; 32],

    /// The registry index of the account.
    pub registry_index: u64,
}

impl RegisteredAccount {
    /// Constructs a new registered account.
    pub fn new(account_key: [u8; 32], registry_index: u64) -> Self {
        Self {
            account_key,
            registry_index,
        }
    }
}

impl PartialEq for RegisteredAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key && self.registry_index == other.registry_index
    }
}

impl Eq for RegisteredAccount {}
