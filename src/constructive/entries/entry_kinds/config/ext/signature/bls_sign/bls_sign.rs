use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::constructive::entry::entry_kinds::config::ext::signature::bls_sign::error::bls_sign_error::ConfigBLSSignError;
use crate::transmutative::bls::sign::bls_sign as bls_sign_message;
use crate::transmutative::key::KeyHolder;

impl Config {
    /// Signs the `Config` signature message (sighash) with BLS secret key.
    pub fn bls_sign(&self, keyholder: &KeyHolder) -> Result<[u8; 96], ConfigBLSSignError> {
        let sighash = self.sighash().map_err(ConfigBLSSignError::SighashError)?;
        let bls_secret_key = keyholder.bls_secret_key();
        Ok(bls_sign_message(bls_secret_key, sighash))
    }
}
