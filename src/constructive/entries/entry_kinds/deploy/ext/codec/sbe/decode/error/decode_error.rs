use crate::constructive::core_types::entities::account::root_account::ext::codec::sbe::decode::error::decode_error::RootAccountSBEDecodeError;
use crate::constructive::core_types::target::ext::codec::sbe::decode::error::decode_error::TargetSBEDecodeError;

/// Errors that can occur when decoding a `Deploy` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeploySBEDecodeError {
    InvalidEntryKindByteError { expected: u8, got: u8 },
    DeploySBEInsufficientBytesForRootAccountLengthPrefix { got_total: usize },
    DeploySBERootAccountLengthPrefixBytesConversionError,
    DeploySBERootAccountLengthPrefixExceedsPayload { root_len: usize, got_after_prefix: usize },
    DeploySBERootAccountDecodeError(RootAccountSBEDecodeError),
    DeploySBEInsufficientBytesForProgramLengthPrefix { got_total: usize },
    DeploySBEProgramLengthPrefixBytesConversionError,
    DeploySBEProgramLengthPrefixExceedsPayload { program_len: usize, got_after_prefix: usize },
    DeploySBEProgramDecompileError,
    DeploySBEInsufficientBytesForInitialBalance { got_total: usize },
    DeploySBEInitialBalanceBytesConversionError,
    DeploySBEInsufficientBytesForTarget { got_total: usize },
    DeploySBETargetDecodeError(TargetSBEDecodeError),
    DeploySBETrailingBytesAfterDeploy { trailing: usize },
}
