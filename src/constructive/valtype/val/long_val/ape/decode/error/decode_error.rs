/// Enum to represent errors that can occur when decoding a `LongVal` from a bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LongValAPEDecodeError {
    TierBitsCollectError,
    ValueBitsCollectError,
    LongValFromCompactBytesConstructionError,
}
