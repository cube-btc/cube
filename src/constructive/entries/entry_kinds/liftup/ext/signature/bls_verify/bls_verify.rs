use crate::constructive::entries::entry_kinds::liftup::ext::signature::bls_verify::error::bls_verify_error::LiftupBLSVerifyError;
use crate::constructive::entries::entry_kinds::liftup::liftup::Liftup;
use crate::transmutative::bls::verify::bls_verify as bls_verify_message;

impl Liftup {
    /// Verifies a BLS signature over this `Liftup`’s signature message (sighash).
    pub fn bls_verify(&self, bls_signature: [u8; 96]) -> Result<(), LiftupBLSVerifyError> {
        // 1 Get the liftup sighash.
        let sighash = self.sighash().map_err(LiftupBLSVerifyError::SighashError)?;

        // 2 Get the BLS public key from the liftup root account.
        let bls_public_key = self.root_account.bls_key();

        // 3 Verify the BLS signature over the sighash.
        match bls_verify_message(&bls_public_key, sighash, bls_signature) {
            // 3.a The BLS signature is valid.
            true => Ok(()),

            // 3.b The BLS signature is invalid.
            false => Err(LiftupBLSVerifyError::InvalidBLSSignatureError),
        }
    }
}
