use crate::constructive::entity::account::{
    registered_account::registered_account::RegisteredAccount,
    unregistered_account::unregistered_account::UnregisteredAccount,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an account; a user of the system.
#[derive(Clone, Serialize, Deserialize)]
pub enum Account {
    // A registered and possibly configured account.
    RegisteredAccount(RegisteredAccount),

    // A fresh, unregistered (thus unranked), and unconfigured account.
    UnregisteredAccount(UnregisteredAccount),
}

impl Account {
    /// Creates a new registered account.
    pub fn new_registered_account(
        key: [u8; 32],
        registery_index: u64,
        rank: Option<u64>,
        bls_key: Option<[u8; 48]>,
        secondary_aggregation_key: Option<Vec<u8>>,
    ) -> Self {
        // 1 Construct the registered account.
        let registered_account = RegisteredAccount::new(
            key,
            registery_index,
            rank,
            bls_key,
            secondary_aggregation_key,
        );

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
            Self::RegisteredAccount(registered_account) => registered_account.key,

            // The account is not registered.
            Self::UnregisteredAccount(unregistered_account) => unregistered_account.key,
        }
    }

    /// Returns the account's rank.
    pub fn rank(&self) -> Option<u64> {
        match self {
            // The account is registered.
            Self::RegisteredAccount(registered_account) => registered_account.rank.to_owned(),

            // The account is not registered.
            Self::UnregisteredAccount(_) => None,
        }
    }

    /// Sets or updates the rank of the account.
    pub fn set_or_update_rank(&mut self, rank: u64) -> bool {
        match self {
            // The account is registered.
            Self::RegisteredAccount(registered_account) => {
                // Update the rank.
                registered_account.rank = Some(rank);

                // Return success.
                true
            }

            // The account is not registered.
            Self::UnregisteredAccount(_) => false,
        }
    }

    /// Returns the account as a JSON object.
    pub fn json(&self) -> Value {
        match self {
            // The account is a registered account.
            Self::RegisteredAccount(registered_account) => registered_account.json(),

            // The account is an unregistered account.
            Self::UnregisteredAccount(unregistered_account) => unregistered_account.json(),
        }
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.account_key() == other.account_key()
    }
}

impl Eq for Account {}
