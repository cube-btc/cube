use crate::constructive::entity::account::{
    account::registered_account::registered_account::RegisteredAccount,
    account::unregistered_account::unregistered_account::UnregisteredAccount,
};
use serde::{Deserialize, Serialize};

/// Represents an account; a user of the system.
#[derive(Clone, Serialize, Deserialize)]
pub enum Account {
    // A registered account.
    RegisteredAccount(RegisteredAccount),

    // A unregistered account.
    UnregisteredAccount(UnregisteredAccount),
}

impl Account {
    /// Creates a new registered account.
    pub fn new_registered_account(account_key: [u8; 32], registery_index: u64) -> Self {
        // 1 Construct the registered account.
        let registered_account = RegisteredAccount::new(account_key, registery_index);

        // 2 Return the registered account.
        Self::RegisteredAccount(registered_account)
    }

    /// Creates a new unregistered account.
    pub fn new_unregistered_account(key: [u8; 32]) -> Self {
        // 1 Construct the unregistered account.
        let unregistered_account = UnregisteredAccount::new(key);

        // 2 Return the unregistered account.
        Self::UnregisteredAccount(unregistered_account)
    }

    /// Returns whether the account is registered.
    pub fn is_registered(&self) -> bool {
        match self {
            // The account is registered.
            Self::RegisteredAccount(_) => true,

            // The account is not registered.
            Self::UnregisteredAccount(_) => false,
        }
    }

    /// Returns the account's secp256k1 public key.
    pub fn account_key(&self) -> [u8; 32] {
        match self {
            // The account is registered.
            Self::RegisteredAccount(registered_account) => registered_account.account_key,

            // The account is not registered.
            Self::UnregisteredAccount(unregistered_account) => unregistered_account.account_key,
        }
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::RegisteredAccount(a), Self::RegisteredAccount(b)) => a == b,
            (Self::UnregisteredAccount(a), Self::UnregisteredAccount(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Account {}
