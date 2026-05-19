/// Errors that can occur when encoding a `Call` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallSBEEncodeError {
    CallSBERootAccountPayloadTooLargeForU32LengthPrefix { len: usize },
    CallSBEContractPayloadTooLargeForU32LengthPrefix { len: usize },
    CallSBECalldataPayloadTooLargeForU32LengthPrefix { len: usize },
}
