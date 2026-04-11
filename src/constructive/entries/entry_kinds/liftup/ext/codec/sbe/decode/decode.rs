use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::txo::lift::lift::Lift;

use super::error::LiftupSBEDecodeError;

impl Liftup {
    /// Decodes a `Liftup` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Liftup, LiftupSBEDecodeError> {
        // 1 Leading byte must be the Liftup SBE variant tag.
        if bytes.is_empty() {
            return Err(
                LiftupSBEDecodeError::LiftupSBEInsufficientBytesForRootAccountLengthPrefix {
                    got_total: 0,
                },
            );
        }
        if bytes[0] != 0x04 {
            return Err(LiftupSBEDecodeError::InvalidEntryKindByteError {
                expected: 0x04,
                got: bytes[0],
            });
        }

        // 2 Ensure there are at least four bytes after the tag for the `RootAccount` blob length prefix.
        if bytes.len() < 5 {
            return Err(
                LiftupSBEDecodeError::LiftupSBEInsufficientBytesForRootAccountLengthPrefix {
                    got_total: bytes.len(),
                },
            );
        }

        // 3 Decode the little-endian `u32` root blob length.
        let root_len_u32 = u32::from_le_bytes(bytes[1..5].try_into().map_err(|_| {
            LiftupSBEDecodeError::LiftupSBERootAccountLengthPrefixBytesConversionError
        })?);
        let root_len = root_len_u32 as usize;

        // 4 Isolate the bytes after the root length prefix.
        let after_root_len_prefix = &bytes[5..];

        // 5 Ensure the buffer holds `root_len` bytes of `RootAccount` SBE after the prefix.
        if after_root_len_prefix.len() < root_len {
            return Err(
                LiftupSBEDecodeError::LiftupSBERootAccountLengthPrefixExceedsPayload {
                    root_len,
                    got_after_prefix: after_root_len_prefix.len(),
                },
            );
        }

        // 6 Split out the `RootAccount` blob and decode it; keep the tail for `Target` and `lift_prevtxos`.
        let (root_slice, after_root_account) = after_root_len_prefix.split_at(root_len);
        let root_account = RootAccount::decode_sbe(root_slice)
            .map_err(LiftupSBEDecodeError::LiftupSBERootAccount)?;

        // 7 Ensure there are at least eight bytes for the `Target` SBE tail.
        if after_root_account.len() < 8 {
            return Err(LiftupSBEDecodeError::LiftupSBEInsufficientBytesForTarget {
                got_total: bytes.len(),
            });
        }

        // 8 Decode the `Target` from the next eight bytes.
        let target = Target::decode_sbe(&after_root_account[..8])
            .map_err(LiftupSBEDecodeError::LiftupSBETarget)?;

        // 9 Isolate the bytes after the `Target` SBE tail.
        let after_target = &after_root_account[8..];

        // 10 Ensure there are at least four bytes for the `lift_prevtxos` count prefix.
        if after_target.len() < 4 {
            return Err(
                LiftupSBEDecodeError::LiftupSBEInsufficientBytesForLiftCountPrefix {
                    got_total: bytes.len(),
                },
            );
        }

        // 11 Decode the little-endian `u32` `lift_prevtxos` count.
        let lift_prevtxos_count = u32::from_le_bytes(
            after_target[0..4]
                .try_into()
                .map_err(|_| LiftupSBEDecodeError::LiftupSBELiftCountPrefixBytesConversionError)?,
        );

        // 12 Isolate the bytes after the count prefix.
        let mut lift_bytes = &after_target[4..];

        // 13 Decode exactly `lift_prevtxos_count` `Lift` values from the remaining bytes.
        let mut lift_prevtxos = Vec::new();
        for index in 0..(lift_prevtxos_count as usize) {
            // 13.a Ensure there is at least one byte to attempt a `Lift` decode.
            if lift_bytes.is_empty() {
                return Err(
                    LiftupSBEDecodeError::LiftupSBEInsufficientBytesForLiftEntry {
                        index,
                        count: lift_prevtxos_count,
                        got_total: bytes.len(),
                    },
                );
            }

            // 13.b Decode one `Lift` and advance by that encoding's byte length.
            let lift = Lift::decode_sbe(lift_bytes)
                .map_err(|source| LiftupSBEDecodeError::LiftupSBELift { index, source })?;
            let consumed = lift.encode_sbe().len();
            if lift_bytes.len() < consumed {
                return Err(
                    LiftupSBEDecodeError::LiftupSBEInsufficientBytesForLiftEntry {
                        index,
                        count: lift_prevtxos_count,
                        got_total: bytes.len(),
                    },
                );
            }
            lift_bytes = &lift_bytes[consumed..];
            lift_prevtxos.push(lift);
        }

        // 14 Reject trailing bytes after the full `Liftup` payload.
        if !lift_bytes.is_empty() {
            return Err(LiftupSBEDecodeError::LiftupSBETrailingBytesAfterLiftup {
                trailing: lift_bytes.len(),
            });
        }

        // 15 Construct and return the `Liftup`.
        Ok(Liftup::new(root_account, target, lift_prevtxos))
    }
}
