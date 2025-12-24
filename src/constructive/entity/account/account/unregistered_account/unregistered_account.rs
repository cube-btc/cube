use serde::{Deserialize, Serialize};

/// A struct for representing an unregistered account.
#[derive(Clone, Serialize, Deserialize)]
pub struct UnregisteredAccount {
    /// The Schnorr public key of the account.
    pub account_key: [u8; 32],
}

impl UnregisteredAccount {
    /// Constructs a new unregistered account.
    pub fn new(account_key: [u8; 32]) -> Self {
        Self { account_key }
    }
}

impl PartialEq for UnregisteredAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key
    }
}

impl Eq for UnregisteredAccount {}
