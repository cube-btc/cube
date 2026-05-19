use crate::constructive::core_types::valtypes::val::atomic_val::ape::encode::error::encode_error::AtomicValAPEEncodeError;

/// Errors that can occur when encoding a `MethodIndex` into an APE bit vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodIndexAPEEncodeError {
    AtomicValAPEEncodeError(AtomicValAPEEncodeError),
    IndexDoesNotFitAtomicEncoding { index: u16, methods_len: usize },
}
