use crate::constructive::core_types::valtypes::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;

/// Errors that can occur when decoding an `OpsPrice` from an APE bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsPriceAPEDecodeError {
    UnexpectedEndOfBitstream,
    LongValAPEDecodeError(LongValAPEDecodeError),
    /// `base_ops_price + overhead` does not fit in a `u64`.
    OpsPriceTotalOverflow {
        base_ops_price: u64,
        overhead: u64,
    },
}
