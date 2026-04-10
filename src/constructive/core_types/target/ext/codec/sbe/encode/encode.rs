use crate::constructive::core_types::target::target::Target;

impl Target {
    /// Encodes this `Target` as Structural Byte-scope Encoding (SBE) bytes.
    ///
    /// Layout: eight bytes, little-endian `u64` `targeted_at_batch_height`.
    pub fn encode_sbe(&self) -> [u8; 8] {
        // 1 Serialize the batch height as a little-endian `u64`.
        self.targeted_at_batch_height.to_le_bytes()
    }
}
