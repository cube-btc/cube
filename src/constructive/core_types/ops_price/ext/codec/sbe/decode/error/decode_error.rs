/// Errors that can occur when decoding an `OpsPrice` from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsPriceSBEDecodeError {
    /// The payload is not exactly 8 bytes (little-endian `u64` `ops_price_ppm`; `got` is the observed length).
    OpsPriceSBEInvalidPayloadLength { got: usize },
}
