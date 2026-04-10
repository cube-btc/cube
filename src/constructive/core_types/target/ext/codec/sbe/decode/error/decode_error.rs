/// Errors that can occur when decoding a `Target` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetSBEDecodeError {
    // Fixed 8-byte payload.
    /// The payload is not exactly 8 bytes (little-endian `u64` batch height; `got` is the observed length).
    TargetSBEInvalidPayloadLength { got: usize },
}
