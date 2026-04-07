use crate::constructive::valtype::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Type alias for the `RootAccount` rank value.
type Rank = u64;

/// Enum to represent errors that can occur when decoding a `RootAccount` from an Airly Payload Encoding (APE) bitstream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RootAccountAPEDecodeError {
    FailedToDecodeRankValueAsLongVal(LongValAPEDecodeError),
    FailedToDecodeRankValueAsShortVal(ShortValAPEDecodeError),
    // Accont key decoding errors.
    AccountKeyBitsLengthError,
    AccountKeyBytesConversionError,
    // Bls key decoding errors.
    BlsKeyBitsLengthError,
    BlsKeyBytesConversionError,
    // Flame config decoding errors.
    FlameConfigPresentBitCollectError,
    FlameConfigLengthBitsCollectError,
    FlameConfigLengthBytesConversionError,
    FlameConfigBitsCollectError,
    FailedToDecodeFlameConfigError,
    // Authorization signature decoding errors.
    AuthorizationSignatureBitsLengthError,
    AuthorizationSignatureBytesConversionError,
    // Root account body retrieval errors.
    FailedToRetrieveRMAccountBodyByRank(Rank),
}
