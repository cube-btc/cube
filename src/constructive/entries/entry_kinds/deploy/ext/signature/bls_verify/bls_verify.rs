use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::constructive::entry::entry_kinds::deploy::ext::signature::bls_verify::error::bls_verify_error::DeployBLSVerifyError;
use crate::transmutative::bls::verify::bls_verify as bls_verify_message;

impl Deploy {
    /// Verifies a BLS signature over this `Deploy`'s signature message (sighash).
    pub fn bls_verify(&self, bls_signature: [u8; 96]) -> Result<(), DeployBLSVerifyError> {
        let sighash = self.sighash().map_err(DeployBLSVerifyError::SighashError)?;
        let bls_public_key = self.root_account.bls_key();
        match bls_verify_message(&bls_public_key, sighash, bls_signature) {
            true => Ok(()),
            false => Err(DeployBLSVerifyError::InvalidBLSSignatureError),
        }
    }
}
