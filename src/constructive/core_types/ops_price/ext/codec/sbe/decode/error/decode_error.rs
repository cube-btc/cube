/// Errors that can occur when decoding an `OpsPrice` from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsPriceSBEDecodeError {
    /// The payload is not exactly 4 bytes (little-endian `u32` `ops_price_ppm`; `got` is the observed length).
    OpsPriceSBEInvalidPayloadLength { got: usize },
}
