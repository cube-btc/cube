/// Errors that can occur when encoding a `Move` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MoveSBEEncodeError {
    MoveSBEFromPayloadTooLargeForU32LengthPrefix { len: usize },
    MoveSBEToPayloadTooLargeForU32LengthPrefix { len: usize },
}
