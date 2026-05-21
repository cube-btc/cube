/// Maximum `Varbytes` payload length (stack and wire format).
pub const MAX_VARBYTES_LEN: usize = 4095;

/// Inclusive `Bytes` payload length bounds.
pub const MIN_BYTES_LEN: usize = 1;
pub const MAX_BYTES_LEN: usize = 256;

/// Errors from [`CalldataElement::validate`](super::calldata_element::CalldataElement::validate).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalldataElementValidationError {
    EmptyBytes,
    BytesLengthOutOfRange { len: usize },
    VarbytesLengthExceedsMax { len: usize },
}
