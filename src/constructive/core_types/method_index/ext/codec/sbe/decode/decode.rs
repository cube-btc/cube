use crate::constructive::core_types::method_index::ext::codec::sbe::decode::error::decode_error::MethodIndexSBEDecodeError;
use crate::constructive::core_types::method_index::method_index::MethodIndex;

impl MethodIndex {
    /// Decodes a `MethodIndex` from SBE bytes produced by [`MethodIndex::encode_sbe`].
    ///
    /// The buffer must be exactly two bytes (little-endian `u16` index).
    pub fn decode_sbe(bytes: &[u8]) -> Result<MethodIndex, MethodIndexSBEDecodeError> {
        let arr: [u8; 2] =
            bytes
                .try_into()
                .map_err(|_| MethodIndexSBEDecodeError::MethodIndexSBEInvalidPayloadLength {
                    got: bytes.len(),
                })?;

        Ok(MethodIndex::new(u16::from_le_bytes(arr)))
    }
}
