/// Errors that can occur when decoding a `MethodIndex` from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MethodIndexSBEDecodeError {
    MethodIndexSBEInvalidPayloadLength { got: usize },
}
