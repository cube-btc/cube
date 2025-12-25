use crate::constructive::entity::account::root_account::root_account::verify_bls_key_authorization_signature;
use crate::{
    constructive::ser::{
        deserialize_bls_key, deserialize_schnorr_signature, serialize_bls_key,
        serialize_schnorr_signature,
    },
    inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig,
    inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredButUnconfiguredRootAccount {
    /// The Schnorr public key of the account.
    pub account_key: [u8; 32],

    /// The registery index of the account.
    pub registery_index: u64,

    /// The BLS public key of the account.
    #[serde(
        serialize_with = "serialize_bls_key",
        deserialize_with = "deserialize_bls_key"
    )]
    pub bls_key_to_be_configured: [u8; 48],

    /// The flame config to be configured.
    pub flame_config_to_be_configured: Option<FMAccountFlameConfig>,

    /// Schnorr signature to authorize the BLS key.
    #[serde(
        serialize_with = "serialize_schnorr_signature",
        deserialize_with = "deserialize_schnorr_signature"
    )]
    pub authorization_signature: [u8; 64],
}

impl RegisteredButUnconfiguredRootAccount {
    /// Constructs a new registered but unconfigured root account.
    pub fn new(
        account_key: [u8; 32],
        registery_index: u64,
        bls_key_to_be_configured: [u8; 48],
        flame_config_to_be_configured: Option<FMAccountFlameConfig>,
        authorization_signature: [u8; 64],
    ) -> Self {
        Self {
            account_key,
            registery_index,
            bls_key_to_be_configured,
            flame_config_to_be_configured,
            authorization_signature,
        }
    }

    /// Verifies the authorization signature.
    pub fn verify_authorization_signature(&self) -> bool {
        verify_bls_key_authorization_signature(
            self.account_key,
            self.bls_key_to_be_configured,
            &self.flame_config_to_be_configured,
            self.authorization_signature,
        )
    }

    /// Checks whether the `RegisteredButUnconfiguredRootAccount` is indeed a valid registered but unconfigured account.
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

                // 2.a.2 Check if the BLS key is indeed not configured.
                if bls_key.is_some() {
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

impl PartialEq for RegisteredButUnconfiguredRootAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key
    }
}

impl Eq for RegisteredButUnconfiguredRootAccount {}
