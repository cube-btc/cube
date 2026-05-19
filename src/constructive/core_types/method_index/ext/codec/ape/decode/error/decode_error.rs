use crate::constructive::core_types::valtypes::val::atomic_val::ape::decode::error::decode_error::AtomicValAPEDecodeError;

/// Errors that can occur when decoding a `MethodIndex` from an APE bit stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodIndexAPEDecodeError {
    AtomicValAPEDecodeError(AtomicValAPEDecodeError),
    U16ValueBitsCollectError,
}
