use crate::constructive::core_types::target::ext::codec::sbe::decode::error::TargetSBEDecodeError;
use crate::constructive::entity::account::account::ext::codec::sbe::decode::error::AccountSBEDecodeError;
use crate::constructive::entity::account::root_account::ext::codec::sbe::decode::error::RootAccountSBEDecodeError;

/// Errors that can occur when decoding a `Move` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MoveSBEDecodeError {
    InvalidEntryKindByteError { expected: u8, got: u8 },
    MoveSBEInsufficientBytesForFromLengthPrefix { got_total: usize },
    MoveSBEFromLengthPrefixBytesConversionError,
    MoveSBEFromLengthPrefixExceedsPayload { from_len: usize, got_after_prefix: usize },
    MoveSBEFromRootAccount(RootAccountSBEDecodeError),
    MoveSBEInsufficientBytesForToLengthPrefix { got_total: usize },
    MoveSBEToLengthPrefixBytesConversionError,
    MoveSBEToLengthPrefixExceedsPayload { to_len: usize, got_after_prefix: usize },
    MoveSBEToAccount(AccountSBEDecodeError),
    MoveSBEInsufficientBytesForAmount { got_total: usize },
    MoveSBEAmountBytesConversionError,
    MoveSBEInsufficientBytesForTarget { got_total: usize },
    MoveSBETarget(TargetSBEDecodeError),
    MoveSBETrailingBytesAfterMove { trailing: usize },
}
