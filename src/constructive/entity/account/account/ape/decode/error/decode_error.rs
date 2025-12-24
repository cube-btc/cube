use crate::constructive::valtype::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Type alias for the `Account` rank value.
type Rank = u64;

/// Enum to represent errors that can occur when decoding an `Account` from an Airly Payload Encoding (APE) bitstream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountAPEDecodeError {
    FailedToDecodeRankValueAsShortVal(ShortValAPEDecodeError),
    FailedToDecodeRankValueAsLongVal(LongValAPEDecodeError),
    PublicKeyBitsLengthError,
    PublicKeyPointFromSliceError,
    KeyAlreadyRegisteredError,
    FailedToLocateAccountGivenRank(Rank),
}
