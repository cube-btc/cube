use crate::constructive::entries::entry_types::liftup::liftup::Liftup;

use super::error::LiftupSBEEncodeError;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Liftup {
    /// Structural Byte-scope Encoding (SBE) for `Liftup`.
    ///
    /// This function encodes a `Liftup` in a non-compact, length-prefixed layout so the variable-width
    /// `RootAccount` blob and each `Lift` blob can be recovered from a single buffer.
    ///
    /// Layout:
    /// 1. Four bytes, little-endian `u32`: byte length of the following `RootAccount` SBE blob (`RootAccount::encode_sbe`).
    /// 2. `RootAccount` SBE bytes.
    /// 3. Eight bytes, little-endian `u64`: `Target::encode_sbe` (`targeted_at_batch_height`).
    /// 4. Four bytes, little-endian `u32`: number of `Lift` entries in `lift_prevtxos`.
    /// 5. Concatenation of each `Lift::encode_sbe()` in order.
    pub fn encode_sbe(&self) -> Result<Bytes, LiftupSBEEncodeError> {
        // 1 Encode the `RootAccount` and ensure its byte length fits a `u32` length prefix.
        let root_bytes = self.root_account.encode_sbe();
        let root_len_u32 = u32::try_from(root_bytes.len()).map_err(|_| {
            LiftupSBEEncodeError::LiftupSBERootAccountPayloadTooLargeForU32LengthPrefix {
                len: root_bytes.len(),
            }
        })?;

        // 2 Ensure the `lift_prevtxos` count fits a `u32` count prefix.
        let count_u32 = u32::try_from(self.lift_prevtxos.len()).map_err(|_| {
            LiftupSBEEncodeError::LiftupSBETooManyLiftsForU32CountPrefix {
                count: self.lift_prevtxos.len(),
            }
        })?;

        // 3 Pre-size the output buffer: root prefix + root blob + target + count + all lift encodings.
        let lifts_len: usize = self.lift_prevtxos.iter().map(|l| l.encode_sbe().len()).sum();
        let mut out = Vec::with_capacity(4 + root_bytes.len() + 8 + 4 + lifts_len);

        // 4 Write the root blob length prefix and the root SBE bytes.
        out.extend_from_slice(&root_len_u32.to_le_bytes());
        out.extend_from_slice(&root_bytes);

        // 5 Write the `Target` SBE tail (8 bytes, little-endian batch height).
        out.extend_from_slice(&self.target.encode_sbe());

        // 6 Write the lift count prefix.
        out.extend_from_slice(&count_u32.to_le_bytes());

        // 7 Append each `Lift` SBE encoding in order.
        for lift in &self.lift_prevtxos {
            out.extend(lift.encode_sbe());
        }

        // 8 Return the completed buffer.
        Ok(out)
    }
}
