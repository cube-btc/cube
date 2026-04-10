use crate::constructive::core_types::target::ext::codec::sbe::decode::error::TargetSBEDecodeError;
use crate::constructive::entity::account::root_account::ext::codec::sbe::decode::error::decode_error::RootAccountSBEDecodeError;
use crate::constructive::txo::lift::lift::ext::codec::sbe::decode::error::decode_error::LiftSBEDecodeError;

/// Enum to represent errors that can occur when decoding a `Liftup` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiftupSBEDecodeError {
    // RootAccount blob length prefix (four bytes).
    /// The buffer ended before the four-byte little-endian `RootAccount` SBE blob length prefix.
    LiftupSBEInsufficientBytesForRootAccountLengthPrefix { got_total: usize },
    /// Failed to assemble the four-byte root blob length prefix from the SBE buffer.
    LiftupSBERootAccountLengthPrefixBytesConversionError,
    /// The length prefix exceeds the remaining bytes after the prefix (`root_len` from prefix, `got_after_prefix` bytes available).
    LiftupSBERootAccountLengthPrefixExceedsPayload {
        root_len: usize,
        got_after_prefix: usize,
    },

    // RootAccount SBE blob.
    /// Decoding the `RootAccount` SBE blob failed.
    LiftupSBERootAccount(RootAccountSBEDecodeError),

    // Target SBE tail (eight bytes).
    /// The buffer ended before the eight-byte `Target` SBE tail after the `RootAccount` blob.
    LiftupSBEInsufficientBytesForTarget { got_total: usize },
    /// Decoding the `Target` SBE tail failed.
    LiftupSBETarget(TargetSBEDecodeError),

    // lift_prevtxos count prefix (four bytes).
    /// The buffer ended before the four-byte little-endian `lift_prevtxos` count prefix.
    LiftupSBEInsufficientBytesForLiftCountPrefix { got_total: usize },
    /// Failed to assemble the four-byte `lift_prevtxos` count prefix from the SBE buffer.
    LiftupSBELiftCountPrefixBytesConversionError,

    // lift_prevtxos entries (concatenated Lift SBE payloads).
    /// Expected another `Lift` SBE value but the buffer was exhausted (`index` / `count` from the prefix).
    LiftupSBEInsufficientBytesForLiftEntry {
        index: usize,
        count: u32,
        got_total: usize,
    },
    /// Decoding one `Lift` in `lift_prevtxos` failed (`index` is the failing entry index).
    LiftupSBELift {
        index: usize,
        source: LiftSBEDecodeError,
    },

    // Trailing bytes.
    /// Bytes remained after decoding `root_account`, `target`, and `count` lifts.
    LiftupSBETrailingBytesAfterLiftup { trailing: usize },
}
