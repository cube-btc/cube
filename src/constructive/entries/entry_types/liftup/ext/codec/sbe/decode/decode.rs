use crate::constructive::entries::entry_types::liftup::liftup::Liftup;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::txo::lift::lift::Lift;

use super::error::LiftupSBEDecodeError;

impl Liftup {
    /// Decodes a `Liftup` from Structural Byte-scope Encoding (SBE) bytes produced by [`Liftup::encode_sbe`].
    ///
    /// This function mirrors the encode layout: root length prefix and blob, `Target`, lift count, then each `Lift`.
    /// Each `Lift` is parsed from the front of the remaining slice; consumption length matches `lift.encode_sbe().len()`.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Liftup, LiftupSBEDecodeError> {
        // 1 Ensure there are at least four bytes for the `RootAccount` blob length prefix.
        if bytes.len() < 4 {
            return Err(LiftupSBEDecodeError::LiftupSBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: bytes.len(),
            });
        }

        // 2 Read the little-endian `u32` root blob length and isolate the following bytes.
        let root_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let after_prefix = &bytes[4..];

        // 3 Ensure the buffer holds `root_len` bytes of `RootAccount` SBE after the prefix.
        if after_prefix.len() < root_len {
            return Err(LiftupSBEDecodeError::LiftupSBERootAccountLengthPrefixExceedsPayload {
                root_len,
                got_after_prefix: after_prefix.len(),
            });
        }

        // 4 Split out the root blob and decode it; keep the tail for `Target` and lifts.
        let (root_slice, rest) = after_prefix.split_at(root_len);
        let root_account =
            RootAccount::decode_sbe(root_slice).map_err(LiftupSBEDecodeError::LiftupSBERootAccount)?;

        // 5 Decode the `Target` from the next 8 bytes.
        if rest.len() < 8 {
            return Err(LiftupSBEDecodeError::LiftupSBEInsufficientBytesForTarget {
                got_total: rest.len(),
            });
        }
        let target = Target::decode_sbe(&rest[..8]).map_err(LiftupSBEDecodeError::LiftupSBETarget)?;
        let rest = &rest[8..];

        // 6 Read the four-byte little-endian lift count prefix.
        if rest.len() < 4 {
            return Err(LiftupSBEDecodeError::LiftupSBEInsufficientBytesForLiftCountPrefix {
                got_total: rest.len(),
            });
        }
        let count = u32::from_le_bytes(rest[0..4].try_into().unwrap());
        let mut rest = &rest[4..];

        // 7 Decode exactly `count` `Lift` values from the remaining bytes.
        let mut lift_prevtxos = Vec::new();
        for index in 0..(count as usize) {
            // 7.a Ensure there is at least one byte to attempt a `Lift` decode.
            if rest.is_empty() {
                return Err(LiftupSBEDecodeError::LiftupSBEInsufficientBytesForLiftEntry { index, count });
            }

            // 7.b Decode one `Lift` and advance by that encoding's byte length.
            let lift =
                Lift::decode_sbe(rest).map_err(|source| LiftupSBEDecodeError::LiftupSBELift { index, source })?;
            let consumed = lift.encode_sbe().len();
            if rest.len() < consumed {
                return Err(LiftupSBEDecodeError::LiftupSBEInsufficientBytesForLiftEntry { index, count });
            }
            rest = &rest[consumed..];
            lift_prevtxos.push(lift);
        }

        // 8 Reject trailing bytes after the full `Liftup` payload.
        if !rest.is_empty() {
            return Err(LiftupSBEDecodeError::LiftupSBETrailingBytesAfterLiftup {
                trailing: rest.len(),
            });
        }

        // 9 Construct and return the `Liftup`.
        Ok(Liftup::new(root_account, target, lift_prevtxos))
    }
}
