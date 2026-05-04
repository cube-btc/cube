use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::constructive::entry::entry_kinds::config::ext::signature::bls_verify::error::bls_verify_error::ConfigBLSVerifyError;
use crate::transmutative::bls::verify::bls_verify as bls_verify_message;

impl Config {
    /// Verifies a BLS signature over this `Config`'s signature message (sighash).
    pub fn bls_verify(&self, bls_signature: [u8; 96]) -> Result<(), ConfigBLSVerifyError> {
        let sighash = self.sighash().map_err(ConfigBLSVerifyError::SighashError)?;
        let bls_public_key = self.root_account.bls_key();
        match bls_verify_message(&bls_public_key, sighash, bls_signature) {
            true => Ok(()),
            false => Err(ConfigBLSVerifyError::InvalidBLSSignatureError),
        }
    }
}
