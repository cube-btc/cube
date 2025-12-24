use crate::constructive::ser::{
    deserialize_bls_key, deserialize_schnorr_signature, serialize_bls_key,
    serialize_schnorr_signature,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredButBLSUnconfiguredRootAccount {
    /// The secp256k1 public key of the account.
    pub account_key: [u8; 32],

    /// The BLS key of the account.
    #[serde(
        serialize_with = "serialize_bls_key",
        deserialize_with = "deserialize_bls_key"
    )]
    pub bls_key_to_be_configured: [u8; 48],

    /// Schnorr signature to prove the authenticity of the BLS key.
    #[serde(
        serialize_with = "serialize_schnorr_signature",
        deserialize_with = "deserialize_schnorr_signature"
    )]
    pub authentication_signature: [u8; 64],
}

impl RegisteredButBLSUnconfiguredRootAccount {
    pub fn new(
        account_key: [u8; 32],
        bls_key_to_be_configured: [u8; 48],
        authentication_signature: [u8; 64],
    ) -> Self {
        Self {
            account_key,
            bls_key_to_be_configured,
            authentication_signature,
        }
    }
}
