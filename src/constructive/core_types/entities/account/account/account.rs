use crate::constructive::core_types::entities::account::account::{
    registered_account::registered_account::RegisteredAccount,
    unregistered_account::unregistered_account::UnregisteredAccount,
};
use crate::inscriptive::registery::registery::REGISTERY;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Represents an account; a user of the system.
#[derive(Clone, Serialize, Deserialize)]
pub enum Account {
    // A registered account.
    RegisteredAccount(RegisteredAccount),

    // A unregistered account.
    UnregisteredAccount(UnregisteredAccount),
}

impl Account {
    /// Returns the `Account` for the given account key from the `Registery`.
    pub async fn account_from_registery(
        account_key: [u8; 32],
        registery: &REGISTERY,
    ) -> Account {
        // 1 Retrieve the account info if it is registered.
        let account_info = {
            // 1.1 Lock the registery.
            let _registery = registery.lock().await;

            // 1.2 Get account info by account key.
            _registery.get_account_info_by_account_key(account_key)
        };

        // 2 Match on whether the account is registered or not.
        match account_info {
            // 2.a The account is registered.
            Some((_, _, registery_index, _)) => {
                // 2.a.1 Construct the `RegisteredAccount`.
                let registered_account = RegisteredAccount::new(account_key, registery_index);

                // 2.a.2 Construct and return the `Account`.
                Self::RegisteredAccount(registered_account)
            }

            // 2.b The account is not registered.
            None => {
                // 2.b.1 Construct the `UnregisteredAccount`.
                let unregistered_account = UnregisteredAccount::new(account_key);

                // 2.b.2 Construct and return the `Account`.
                Self::UnregisteredAccount(unregistered_account)
            }
        }
    }

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

    /// Returns the account as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the JSON object.
        let mut obj = Map::new();

        // 2 Match on account variant.
        match self {
            // 2.a The `Account` is registered.
            Self::RegisteredAccount(registered_account) => {
                obj.insert("kind".to_string(), Value::String("registered".to_string()));
                obj.insert(
                    "account_key".to_string(),
                    Value::String(hex::encode(registered_account.account_key)),
                );
                obj.insert(
                    "registery_index".to_string(),
                    Value::Number(registered_account.registery_index.into()),
                );
            }

            // 2.b The `Account` is unregistered.
            Self::UnregisteredAccount(unregistered_account) => {
                obj.insert("kind".to_string(), Value::String("unregistered".to_string()));
                obj.insert(
                    "account_key_to_be_registered".to_string(),
                    Value::String(hex::encode(unregistered_account.account_key_to_be_registered)),
                );
            }
        }

        // 3 Return the JSON object.
        Value::Object(obj)
    }

    /// Returns the account's secp256k1 public key.
    pub fn account_key(&self) -> [u8; 32] {
        match self {
            // The account is registered.
            Self::RegisteredAccount(registered_account) => registered_account.account_key,

            // The account is not registered.
            Self::UnregisteredAccount(unregistered_account) => {
                unregistered_account.account_key_to_be_registered
            }
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
