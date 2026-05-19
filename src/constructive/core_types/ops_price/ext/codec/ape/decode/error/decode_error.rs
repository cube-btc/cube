use crate::constructive::core_types::valtypes::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Errors that can occur when decoding an `OpsPrice` from an APE bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsPriceAPEDecodeError {
    UnexpectedEndOfBitstream,
    ShortValAPEDecodeError(ShortValAPEDecodeError),
    /// `base_ops_price + overhead` does not fit in a `u32`.
    OpsPriceTotalOverflow {
        base_ops_price: u32,
        overhead: u32,
    },
}
