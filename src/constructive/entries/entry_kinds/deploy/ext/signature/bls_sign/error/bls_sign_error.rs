use crate::constructive::entry::entry_kinds::deploy::ext::signature::sighash::error::sighash_error::DeploySighashError;

/// Errors associated with signing a `Deploy` with BLS.
#[derive(Debug, Clone)]
pub enum DeployBLSSignError {
    SighashError(DeploySighashError),
}
