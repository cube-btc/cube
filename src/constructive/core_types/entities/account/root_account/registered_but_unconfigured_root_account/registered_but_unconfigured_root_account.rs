use crate::constructive::entity::account::root_account::root_account::verify_bls_key_authorization_signature;
use crate::{
    transmutative::bls::bls_ser::{
        deserialize_bls_key, deserialize_schnorr_signature, serialize_bls_key,
        serialize_schnorr_signature,
    },
    inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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

    /// Validates the `RegisteredButUnconfiguredRootAccount`'s keys.
    pub fn validate_bls_key(&self) -> bool {
        // 1 Verify that the BLS key is indeed a valid BLS public key: TODO.

        // 2 Return true.
        true
    }

    /// Returns the registered but unconfigured root account as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "kind".to_string(),
            Value::String("registered_but_unconfigured".to_string()),
        );

        obj.insert(
            "account_key".to_string(),
            Value::String(hex::encode(self.account_key)),
        );

        obj.insert(
            "registery_index".to_string(),
            Value::String(self.registery_index.to_string()),
        );

        obj.insert(
            "bls_key_to_be_configured".to_string(),
            Value::String(hex::encode(self.bls_key_to_be_configured)),
        );

        obj.insert(
            "flame_config_to_be_configured".to_string(),
            match &self.flame_config_to_be_configured {
                Some(flame_config) => flame_config.json(),
                None => Value::Null,
            },
        );

        obj.insert(
            "authorization_signature".to_string(),
            Value::String(hex::encode(self.authorization_signature)),
        );

        Value::Object(obj)
    }
}

impl PartialEq for RegisteredButUnconfiguredRootAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key
    }
}

impl Eq for RegisteredButUnconfiguredRootAccount {}
