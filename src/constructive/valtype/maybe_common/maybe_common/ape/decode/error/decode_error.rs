use crate::constructive::valtype::maybe_common::common::common_long::ape::decode::error::decode_error::CommonLongValAPEDecodeError;
use crate::constructive::valtype::maybe_common::common::common_short::ape::decode::error::decode_error::CommonShortValAPEDecodeError;
use crate::constructive::valtype::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Enum to represent errors that can occur when decoding a `MaybeCommon` from a bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaybeCommonAPEDecodeError {
    IsCommonBitCollectError,
    CommonShortValAPEDecodeError(CommonShortValAPEDecodeError),
    CommonLongValAPEDecodeError(CommonLongValAPEDecodeError),
    ShortValAPEDecodeError(ShortValAPEDecodeError),
    LongValAPEDecodeError(LongValAPEDecodeError),
}
