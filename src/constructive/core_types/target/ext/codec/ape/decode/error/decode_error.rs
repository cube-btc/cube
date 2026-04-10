/// Enum to represent errors that can occur when decoding a `Target` from an Airly Payload Encoding (APE) bitstream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetAPEDecodeError {
    UnexpectedEndOfBitstream,
    TargetBatchHeightUnderflow,
}
