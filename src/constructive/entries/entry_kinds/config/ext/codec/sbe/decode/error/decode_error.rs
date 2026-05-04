use crate::constructive::core_types::entities::account::root_account::ext::codec::sbe::decode::error::decode_error::RootAccountSBEDecodeError;
use crate::constructive::core_types::target::ext::codec::sbe::decode::error::decode_error::TargetSBEDecodeError;

/// Errors that can occur when decoding a `Config` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigSBEDecodeError {
    InvalidEntryKindByteError { expected: u8, got: u8 },
    ConfigSBEInsufficientBytesForRootAccountLengthPrefix { got_total: usize },
    ConfigSBERootAccountLengthPrefixBytesConversionError,
    ConfigSBERootAccountLengthPrefixExceedsPayload { root_len: usize, got_after_prefix: usize },
    ConfigSBERootAccountDecodeError(RootAccountSBEDecodeError),
    ConfigSBEInsufficientBytesForSecondaryAggregationPresenceFlag { got_total: usize },
    ConfigSBEInsufficientBytesForSecondaryAggregationLengthPrefix { got_total: usize },
    ConfigSBESecondaryAggregationLengthPrefixBytesConversionError,
    ConfigSBESecondaryAggregationLengthPrefixExceedsPayload { key_len: usize, got_after_prefix: usize },
    ConfigSBESecondaryAggregationPresenceLengthMismatch { key_len: usize },
    ConfigSBEInsufficientBytesForProjectorConfigPresenceFlag { got_total: usize },
    ConfigSBEInsufficientBytesForProjectorConfigPayload { got_total: usize },
    ConfigSBEProjectorConfigPresenceMismatch,
    ConfigSBEInsufficientBytesForFlameConfigPresenceFlag { got_total: usize },
    ConfigSBEInsufficientBytesForFlameConfigLengthPrefix { got_total: usize },
    ConfigSBEFlameConfigLengthPrefixBytesConversionError,
    ConfigSBEFlameConfigLengthPrefixExceedsPayload { flame_len: usize, got_after_prefix: usize },
    ConfigSBEFlameConfigDecodeError,
    ConfigSBEFlameConfigPresenceLengthMismatch { flame_len: usize },
    ConfigSBEInsufficientBytesForTarget { got_total: usize },
    ConfigSBETargetDecodeError(TargetSBEDecodeError),
    ConfigSBETrailingBytesAfterConfig { trailing: usize },
}
