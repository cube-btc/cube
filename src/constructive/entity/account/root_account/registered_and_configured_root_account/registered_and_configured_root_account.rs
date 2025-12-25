use crate::constructive::ser::{deserialize_bls_key, serialize_bls_key};
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
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

    /// Checks whether the `RegisteredAndConfiguredRootAccount` is indeed a valid registered and configured account.
    pub async fn validate(&self, registery_manager: &REGISTERY_MANAGER) -> bool {
        // 1 Get account info by account key.
        let account_info = {
            // 1.1 Lock the registery manager.
            let _registery_manager = registery_manager.lock().await;

            // 1.2 Get account info by account key.
            _registery_manager.get_account_info_by_account_key(self.account_key)
        };

        // 2 Check if the account is already registered.
        match account_info {
            // 2.a The account is indeed registered.
            Some((_, bls_key, registery_index, _)) => {
                // 2.a.1 Check if the registery index is the same.
                if registery_index != self.registery_index {
                    return false;
                }

                // 2.a.2 Check if the BLS key is the same.
                if bls_key != Some(self.bls_key) {
                    return false;
                }

                // 2.a.3 Return true.
                true
            }

            // 2.b The account is not registered.
            None => false,
        }
    }
}

impl PartialEq for RegisteredAndConfiguredRootAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key
    }
}

impl Eq for RegisteredAndConfiguredRootAccount {}
