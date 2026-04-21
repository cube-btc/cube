use crate::constructive::entries::entry_kinds::r#move::ext::signature::bls_verify::error::bls_verify_error::MoveBLSVerifyError;
use crate::constructive::entries::entry_kinds::r#move::r#move::Move;
use crate::transmutative::bls::verify::bls_verify as bls_verify_message;

impl Move {
    /// Verifies a BLS signature over this `Move`'s signature message (sighash).
    pub fn bls_verify(&self, bls_signature: [u8; 96]) -> Result<(), MoveBLSVerifyError> {
        // 1 Get the move sighash.
        let sighash = self.sighash().map_err(MoveBLSVerifyError::SighashError)?;

        // 2 Get the BLS public key from the sender root account.
        let bls_public_key = self.from.bls_key();

        // 3 Verify the BLS signature over the sighash.
        match bls_verify_message(&bls_public_key, sighash, bls_signature) {
            // 3.a The BLS signature is valid.
            true => Ok(()),

            // 3.b The BLS signature is invalid.
            false => Err(MoveBLSVerifyError::InvalidBLSSignatureError),
        }
    }
}
