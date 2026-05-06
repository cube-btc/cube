use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::constructive::entry::entry_kinds::deploy::ext::signature::bls_sign::error::bls_sign_error::DeployBLSSignError;
use crate::transmutative::bls::sign::bls_sign as bls_sign_message;
use crate::transmutative::key::KeyHolder;

impl Deploy {
    /// Signs the `Deploy` signature message (sighash) with BLS secret key.
    pub fn bls_sign(&self, keyholder: &KeyHolder) -> Result<[u8; 96], DeployBLSSignError> {
        let sighash = self.sighash().map_err(DeployBLSSignError::SighashError)?;
        let bls_secret_key = keyholder.bls_secret_key();
        Ok(bls_sign_message(bls_secret_key, sighash))
    }
}
