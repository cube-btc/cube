use crate::constructive::core_types::target::target::Target;

use super::error::TargetSBEDecodeError;

impl Target {
    /// Decodes a `Target` from Structural Byte-scope Encoding (SBE) bytes produced by [`Target::encode_sbe`].
    ///
    /// The buffer must be exactly eight bytes (little-endian `u64` batch height).
    pub fn decode_sbe(bytes: &[u8]) -> Result<Target, TargetSBEDecodeError> {
        // 1 Require exactly eight bytes for the little-endian `u64`.
        let arr: [u8; 8] = bytes.try_into().map_err(|_| TargetSBEDecodeError::TargetSBEInvalidPayloadLength {
            got: bytes.len(),
        })?;

        // 2 Decode the height and construct the `Target`.
        Ok(Target::new(u64::from_le_bytes(arr)))
    }
}
