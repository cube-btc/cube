use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;

use super::error::LiftupSBEEncodeError;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Liftup {
    /// Structural Byte-scope Encoding (SBE) encoding for `Liftup`.
    pub fn encode_sbe(&self) -> Result<Bytes, LiftupSBEEncodeError> {
        // 1 Encode the `RootAccount` SBE blob.
        let root_bytes = self.root_account.encode_sbe();

        // 2 Ensure the `RootAccount` SBE payload length fits a `u32` length prefix.
        let root_len_u32 = u32::try_from(root_bytes.len()).map_err(|_| {
            LiftupSBEEncodeError::LiftupSBERootAccountPayloadTooLargeForU32LengthPrefix {
                len: root_bytes.len(),
            }
        })?;

        // 3 Ensure the `lift_prevtxos` count fits a `u32` count prefix.
        let lift_prevtxos_count_u32 = u32::try_from(self.lift_prevtxos.len()).map_err(|_| {
            LiftupSBEEncodeError::LiftupSBETooManyLiftsForU32CountPrefix {
                count: self.lift_prevtxos.len(),
            }
        })?;

        // 4 Initialize the byte vector.
        let mut bytes = Bytes::new();

        // 5 Push 0x04 to indicate that this is a `Liftup` entry.
        bytes.push(0x04);

        // 6 Write the four-byte little-endian `RootAccount` blob length prefix.
        bytes.extend_from_slice(&root_len_u32.to_le_bytes());

        // 7 Encode the `RootAccount` SBE bytes.
        bytes.extend_from_slice(&root_bytes);

        // 8 Encode the `Target` SBE tail (eight bytes, little-endian batch height).
        bytes.extend_from_slice(&self.target.encode_sbe());

        // 9 Encode the four-byte little-endian `lift_prevtxos` count prefix.
        bytes.extend_from_slice(&lift_prevtxos_count_u32.to_le_bytes());

        // 10 Encode each `Lift` SBE payload in order.
        for lift in &self.lift_prevtxos {
            bytes.extend(lift.encode_sbe());
        }

        // 11 Return the bytes.
        Ok(bytes)
    }
}
