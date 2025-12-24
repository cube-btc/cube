use serde::{Deserialize, Serialize};

/// A registered account.
#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredAccount {
    /// The Schnorr public key of the account.
    pub account_key: [u8; 32],

    /// The registery index of the account.
    pub registery_index: u64,
}

impl RegisteredAccount {
    /// Constructs a new registered account.
    pub fn new(account_key: [u8; 32], registery_index: u64) -> Self {
        Self {
            account_key,
            registery_index,
        }
    }
}

impl PartialEq for RegisteredAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key && self.registery_index == other.registery_index
    }
}

impl Eq for RegisteredAccount {}
