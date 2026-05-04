use crate::constructive::entry::entry_kinds::config::ext::codec::sbe::encode::error::ConfigSBEEncodeError;

/// Errors associated with generating a sighash for a `Config`.
#[derive(Debug, Clone)]
pub enum ConfigSighashError {
    SBEEncodeError(ConfigSBEEncodeError),
}
