use crate::constructive::calldata::element::sbe::encode::error::encode_error::CalldataElementSBEEncodeError;

/// Errors that can occur when encoding a `Call` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallSBEEncodeError {
    CallSBERootAccountPayloadTooLargeForU32LengthPrefix { len: usize },
    CallSBEContractPayloadTooLargeForU32LengthPrefix { len: usize },
    CalldataElementSBEEncodeError(CalldataElementSBEEncodeError),
    CallSBECalldataPayloadTooLargeForU32LengthPrefix { len: usize },
}
