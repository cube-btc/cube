use crate::constructive::valtype::val::{
    long_val::ape::decode::error::decode_error::LongValAPEDecodeError,
    short_val::ape::decode::error::decode_error::ShortValAPEDecodeError,
};

/// Type alias for the `Contract` rank value.
type Rank = u64;

/// Enum to represent errors that can occur when decoding a `Contract` from an Airly Payload Encoding (APE) bitstream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContractAPEDecodeError {
    FailedToDecodeRankValueAsLongVal(LongValAPEDecodeError),
    FailedToDecodeRankValueAsShortVal(ShortValAPEDecodeError),
    ContractNotFoundInRegisteryManagerWithRank(Rank),
}
