/// Enum to represent errors that can occur when encoding a `Swapout` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwapoutSBEEncodeError {
    /// The `RootAccount` SBE payload is larger than `u32::MAX` and cannot be length-prefixed (`len` is its byte length).
    SwapoutSBERootAccountPayloadTooLargeForU32LengthPrefix { len: usize },
}
