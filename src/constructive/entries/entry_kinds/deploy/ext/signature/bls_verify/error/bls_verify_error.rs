use crate::constructive::entry::entry_kinds::deploy::ext::signature::sighash::error::sighash_error::DeploySighashError;

/// Errors associated with verifying a BLS signature over a `Deploy`.
#[derive(Debug, Clone)]
pub enum DeployBLSVerifyError {
    SighashError(DeploySighashError),
    InvalidBLSSignatureError,
}
