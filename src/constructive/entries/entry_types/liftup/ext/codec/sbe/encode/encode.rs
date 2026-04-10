use crate::constructive::entry::entry_types::liftup::liftup::Liftup;

use super::error::LiftupSBEEncodeError;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Liftup {
    /// Structural Byte-scope Encoding (SBE) encoding for `Liftup`.
    ///
    /// This function encodes a `Liftup` in non-compact, byte-scope format akin to bincode-style layouts
    /// (see also [`Liftup::decode_sbe`]).
    ///
    /// Layout (little-endian for multi-byte integers):
    ///
    /// - four-byte `u32` length of the following `RootAccount` SBE blob,
    /// - `RootAccount` SBE bytes ([`crate::constructive::entity::account::root_account::root_account::RootAccount::encode_sbe`]),
    /// - eight-byte `Target` SBE tail,
    /// - four-byte `u32` count of `lift_prevtxos`,
    /// - each `Lift` SBE payload in order ([`crate::constructive::txo::lift::lift::Lift::encode_sbe`]).
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

        // 5 Write the four-byte little-endian `RootAccount` blob length prefix.
        bytes.extend_from_slice(&root_len_u32.to_le_bytes());

        // 6 Encode the `RootAccount` SBE bytes.
        bytes.extend_from_slice(&root_bytes);

        // 7 Encode the `Target` SBE tail (eight bytes, little-endian batch height).
        bytes.extend_from_slice(&self.target.encode_sbe());

        // 8 Encode the four-byte little-endian `lift_prevtxos` count prefix.
        bytes.extend_from_slice(&lift_prevtxos_count_u32.to_le_bytes());

        // 9 Encode each `Lift` SBE payload in order.
        for lift in &self.lift_prevtxos {
            bytes.extend(lift.encode_sbe());
        }

        // 10 Return the bytes.
        Ok(bytes)
    }
}
