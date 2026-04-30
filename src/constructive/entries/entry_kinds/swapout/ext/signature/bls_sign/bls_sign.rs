use crate::constructive::entry::entry_kinds::swapout::ext::signature::bls_sign::error::bls_sign_error::SwapoutBLSSignError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::transmutative::bls::sign::bls_sign as bls_sign_message;
use crate::transmutative::key::KeyHolder;

impl Swapout {
    /// Signs the `Swapout` signature message (sighash) with BLS secret key.
    pub fn bls_sign(&self, keyholder: &KeyHolder) -> Result<[u8; 96], SwapoutBLSSignError> {
        let sighash = self.sighash().map_err(SwapoutBLSSignError::SighashError)?;
        let bls_secret_key = keyholder.bls_secret_key();
        Ok(bls_sign_message(bls_secret_key, sighash))
    }
}
