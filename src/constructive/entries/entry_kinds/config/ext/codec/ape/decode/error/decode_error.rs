use crate::constructive::core_types::entities::account::root_account::ext::codec::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::core_types::target::ext::codec::ape::decode::error::decode_error::TargetAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Airly Payload Encoding (APE) decoding error for `Config`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigAPEDecodeError {
    RootAccountAPEDecodeError(RootAccountAPEDecodeError),
    SecondaryAggregationKeyPresenceBitCollectError,
    SecondaryAggregationKeyLenDecodeError(ShortValAPEDecodeError),
    SecondaryAggregationKeyBitsCollectError,
    ProjectorConfigPresenceBitCollectError,
    ProjectorConfigBitsCollectError,
    ProjectorConfigBytesConversionError,
    FlameConfigPresenceBitCollectError,
    FlameConfigLenDecodeError(ShortValAPEDecodeError),
    FlameConfigBitsCollectError,
    FlameConfigDecodeError,
    TargetAPEDecodeError(TargetAPEDecodeError),
}
