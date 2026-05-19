use crate::constructive::core_types::method_index::method_index::MethodIndex;

impl MethodIndex {
    /// Encodes a `MethodIndex` as Structural Byte-scope Encoding (SBE) bytes.
    ///
    /// Layout: two bytes, little-endian `u16` index.
    pub fn encode_sbe(&self) -> [u8; 2] {
        self.index.to_le_bytes()
    }
}
