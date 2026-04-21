use crate::constructive::entries::entry_kinds::r#move::ext::signature::bls_sign::error::bls_sign_error::MoveBLSSignError;
use crate::constructive::entries::entry_kinds::r#move::r#move::Move;
use crate::transmutative::bls::sign::bls_sign as bls_sign_message;
use crate::transmutative::key::KeyHolder;

impl Move {
    /// Signs the `Move` signature message (sighash) with BLS secret key.
    pub fn bls_sign(&self, keyholder: &KeyHolder) -> Result<[u8; 96], MoveBLSSignError> {
        // 1 Get the move sighash.
        let sighash = self.sighash().map_err(MoveBLSSignError::SighashError)?;

        // 2 Get the BLS secret key from the keyholder.
        let bls_secret_key = keyholder.bls_secret_key();

        // 3 Sign the move sighash with the BLS secret key.
        let bls_signature = bls_sign_message(bls_secret_key, sighash);

        // 4 Return the BLS signature.
        Ok(bls_signature)
    }
}
