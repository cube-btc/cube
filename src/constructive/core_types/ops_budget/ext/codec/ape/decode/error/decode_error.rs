use crate::constructive::core_types::valtypes::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Errors that can occur when decoding an `OpsBudget` from an APE bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsBudgetAPEDecodeError {
    UnexpectedEndOfBitstream,
    ShortValAPEDecodeError(ShortValAPEDecodeError),
}
