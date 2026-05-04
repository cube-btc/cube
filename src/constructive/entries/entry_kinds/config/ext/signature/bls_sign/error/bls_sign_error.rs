use crate::constructive::entry::entry_kinds::config::ext::signature::sighash::error::sighash_error::ConfigSighashError;

/// Errors associated with signing a `Config` with BLS.
#[derive(Debug, Clone)]
pub enum ConfigBLSSignError {
    SighashError(ConfigSighashError),
}
