/// Enum to represent errors that can occur when decoding a `ShortVal` from a bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShortValAPEDecodeError {
    TierBitsCollectError,
    ValueBitsCollectError,
    ShortValFromCompactBytesConstructionError,
}
