/// Errors that can occur when encoding a `Deploy` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeploySBEEncodeError {
    DeploySBERootAccountPayloadTooLargeForU32LengthPrefix { len: usize },
    DeploySBEProgramCompileError,
    DeploySBEProgramPayloadTooLargeForU32LengthPrefix { len: usize },
}
