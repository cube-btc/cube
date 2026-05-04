use crate::constructive::entry::entry_kinds::config::ext::signature::sighash::error::sighash_error::ConfigSighashError;

/// Errors associated with verifying a BLS signature over a `Config`.
#[derive(Debug, Clone)]
pub enum ConfigBLSVerifyError {
    SighashError(ConfigSighashError),
    InvalidBLSSignatureError,
}
