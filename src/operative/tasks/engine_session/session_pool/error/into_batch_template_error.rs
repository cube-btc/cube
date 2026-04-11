use crate::constructive::entry::entry::ext::codec::ape::encode::error::encode_error::EntryAPEEncodeError;

/// Errors associated with converting the `ExecCtx` into a `BatchTemplate`.
#[derive(Debug, Clone)]
pub enum IntoBatchTemplateError {
    EntryAPEEncodeError(EntryAPEEncodeError),
    AggregateBLSSignatureError,
}
