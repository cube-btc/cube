use crate::{
    constructive::ser::{
        deserialize_bls_key, deserialize_schnorr_signature, serialize_bls_key,
        serialize_schnorr_signature,
    },
    inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredButUnconfiguredRootAccount {
    /// The Schnorr public key of the account.
    pub account_key: [u8; 32],

    /// The BLS public key of the account.
    #[serde(
        serialize_with = "serialize_bls_key",
        deserialize_with = "deserialize_bls_key"
    )]
    pub bls_key_to_be_configured: [u8; 48],

    /// The flame config to be configured.
    pub flame_config_to_be_configured: Option<FMAccountFlameConfig>,

    /// Schnorr signature to prove the authenticity of the BLS key.
    #[serde(
        serialize_with = "serialize_schnorr_signature",
        deserialize_with = "deserialize_schnorr_signature"
    )]
    pub authentication_signature: [u8; 64],
}

impl RegisteredButUnconfiguredRootAccount {
    pub fn new(
        account_key: [u8; 32],
        bls_key_to_be_configured: [u8; 48],
        flame_config_to_be_configured: Option<FMAccountFlameConfig>,
        authentication_signature: [u8; 64],
    ) -> Self {
        Self {
            account_key,
            bls_key_to_be_configured,
            flame_config_to_be_configured,
            authentication_signature,
        }
    }
}

impl PartialEq for RegisteredButUnconfiguredRootAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key == other.account_key
    }
}

impl Eq for RegisteredButUnconfiguredRootAccount {}
