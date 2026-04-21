use crate::constructive::entry::entry_kinds::r#move::r#move::Move;

use super::error::encode_error::MoveSBEEncodeError;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Move {
    /// Structural Byte-scope Encoding (SBE) encoding for `Move`.
    pub fn encode_sbe(&self) -> Result<Bytes, MoveSBEEncodeError> {
        // 1 Encode the sender and receiver payloads.
        let from_bytes = self.from.encode_sbe();
        let to_bytes = self.to.encode_sbe();

        // 2 Ensure payload lengths fit `u32` length prefixes.
        let from_len_u32 = u32::try_from(from_bytes.len()).map_err(|_| {
            MoveSBEEncodeError::MoveSBEFromPayloadTooLargeForU32LengthPrefix {
                len: from_bytes.len(),
            }
        })?;
        let to_len_u32 = u32::try_from(to_bytes.len()).map_err(|_| {
            MoveSBEEncodeError::MoveSBEToPayloadTooLargeForU32LengthPrefix {
                len: to_bytes.len(),
            }
        })?;

        // 3 Initialize bytes and write layout.
        let mut bytes = Bytes::new();
        bytes.push(0x00);
        bytes.extend_from_slice(&from_len_u32.to_le_bytes());
        bytes.extend_from_slice(&from_bytes);
        bytes.extend_from_slice(&to_len_u32.to_le_bytes());
        bytes.extend_from_slice(&to_bytes);
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend_from_slice(&self.target.encode_sbe());

        // 4 Return bytes.
        Ok(bytes)
    }
}
