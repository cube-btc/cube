use crate::transmutative::secp::schnorr::Bytes32;
use serde::{Deserialize, Serialize};

/// A struct for representing an unregistered account.
#[derive(Clone, Serialize, Deserialize)]
pub struct UnregisteredAccount {
    /// The Schnorr public key of the account.
    pub account_key_to_be_registered: [u8; 32],
}

impl UnregisteredAccount {
    /// Constructs a new unregistered account.
    pub fn new(account_key: [u8; 32]) -> Self {
        Self {
            account_key_to_be_registered: account_key,
        }
    }

    /// Validates the account key is indeed a valid Schnorr public key.
    pub fn validate_schnorr_key(&self) -> bool {
        // 1 Verify that the account key is indeed a valid Schnorr public key.
        if !self.account_key_to_be_registered.to_even_point().is_none() {
            return false;
        }

        // 2 Return true.
        true
    }
}

impl PartialEq for UnregisteredAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key_to_be_registered == other.account_key_to_be_registered
    }
}

impl Eq for UnregisteredAccount {}
