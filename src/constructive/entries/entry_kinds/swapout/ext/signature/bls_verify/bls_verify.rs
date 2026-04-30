use crate::constructive::entry::entry_kinds::swapout::ext::signature::bls_verify::error::bls_verify_error::SwapoutBLSVerifyError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::transmutative::bls::verify::bls_verify as bls_verify_message;

impl Swapout {
    /// Verifies a BLS signature over this `Swapout`’s signature message (sighash).
    pub fn bls_verify(&self, bls_signature: [u8; 96]) -> Result<(), SwapoutBLSVerifyError> {
        let sighash = self.sighash().map_err(SwapoutBLSVerifyError::SighashError)?;
        let bls_public_key = self.root_account.bls_key();
        match bls_verify_message(&bls_public_key, sighash, bls_signature) {
            true => Ok(()),
            false => Err(SwapoutBLSVerifyError::InvalidBLSSignatureError),
        }
    }
}
