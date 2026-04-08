use crate::constructive::entity::account::root_account::root_account::verify_bls_key_authorization_signature;
use crate::constructive::ser::{
    deserialize_bls_key, deserialize_schnorr_signature, serialize_bls_key,
    serialize_schnorr_signature,
};
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use crate::transmutative::secp::schnorr::Bytes32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UnregisteredRootAccount {
    /// The Schnorr public key of the account.
    pub account_key_to_be_registered: [u8; 32],

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

impl UnregisteredRootAccount {
    /// Constructs a new unregistered root account.
    pub fn new(
        account_key_to_be_registered: [u8; 32],
        bls_key_to_be_configured: [u8; 48],
        flame_config_to_be_configured: Option<FMAccountFlameConfig>,
        authorization_signature: [u8; 64],
    ) -> Self {
        Self {
            account_key_to_be_registered,
            bls_key_to_be_configured,
            flame_config_to_be_configured,
            authorization_signature,
        }
    }

    /// Verifies the authorization signature.
    pub fn verify_authorization_signature(&self) -> bool {
        verify_bls_key_authorization_signature(
            self.account_key_to_be_registered,
            self.bls_key_to_be_configured,
            &self.flame_config_to_be_configured,
            self.authorization_signature,
        )
    }

    /// Validates the `UnregisteredRootAccount`'s Schnorr and BLS keys.
    pub fn validate_schnorr_and_bls_key(&self) -> bool {
        // 1 Verify that the account key is indeed a valid Schnorr public key.
        if !self.account_key_to_be_registered.to_even_point().is_none() {
            return false;
        }

        // 2 Verify that the BLS key is indeed a valid BLS public key: TODO.

        // 3 Return true.
        true
    }
}

impl PartialEq for UnregisteredRootAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account_key_to_be_registered == other.account_key_to_be_registered
    }
}

impl Eq for UnregisteredRootAccount {}
