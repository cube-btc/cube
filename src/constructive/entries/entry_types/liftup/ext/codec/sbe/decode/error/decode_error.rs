use crate::constructive::entity::account::root_account::ext::codec::sbe::decode::error::RootAccountSBEDecodeError;
use crate::constructive::core_types::target::ext::codec::sbe::decode::error::TargetSBEDecodeError;
use crate::constructive::txo::lift::lift::ext::codec::sbe::decode::error::decode_error::LiftSBEDecodeError;

/// Errors that can occur when decoding a `Liftup` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiftupSBEDecodeError {
    // RootAccount length prefix and blob.
    /// The buffer ended before the 4-byte little-endian `RootAccount` SBE blob length prefix.
    LiftupSBEInsufficientBytesForRootAccountLengthPrefix { got_total: usize },
    /// The length prefix exceeds the remaining bytes after the prefix (`root_len` from prefix, `got_after_prefix` bytes available).
    LiftupSBERootAccountLengthPrefixExceedsPayload {
        root_len: usize,
        got_after_prefix: usize,
    },
    /// Decoding the `RootAccount` SBE blob failed.
    LiftupSBERootAccount(RootAccountSBEDecodeError),

    // Target (8-byte SBE).
    /// Fewer than 8 bytes remained for `Target` after the `RootAccount` blob.
    LiftupSBEInsufficientBytesForTarget { got_total: usize },
    /// Decoding the `Target` SBE tail failed.
    LiftupSBETarget(TargetSBEDecodeError),

    // `lift_prevtxos` count and entries.
    /// The buffer ended before the 4-byte little-endian `lift_prevtxos` count prefix.
    LiftupSBEInsufficientBytesForLiftCountPrefix { got_total: usize },
    /// Expected another `Lift` SBE value but the buffer was exhausted (`index` / `count` from the prefix).
    LiftupSBEInsufficientBytesForLiftEntry { index: usize, count: u32 },
    /// Decoding one `Lift` in `lift_prevtxos` failed (`index` is the failing entry index).
    LiftupSBELift {
        index: usize,
        source: LiftSBEDecodeError,
    },
    /// Bytes remained after decoding `root_account`, `target`, and `count` lifts.
    LiftupSBETrailingBytesAfterLiftup { trailing: usize },
}
