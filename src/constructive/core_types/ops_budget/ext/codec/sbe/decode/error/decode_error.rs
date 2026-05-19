/// Errors that can occur when decoding an `OpsBudget` from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsBudgetSBEDecodeError {
    /// The payload is not exactly 5 bytes (presence + little-endian `u32`; `got` is the observed length).
    OpsBudgetSBEInvalidPayloadLength { got: usize },
    /// The presence flag is not `0` or `1`.
    OpsBudgetSBEInvalidPresenceFlag { got: u8 },
}
