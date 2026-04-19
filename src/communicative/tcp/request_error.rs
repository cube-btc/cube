use crate::communicative::tcp::tcp::TCPError;

/// Errors from the generic TCP client (`ping`, liftup v1, …).
#[derive(Debug, Copy, Clone)]
pub enum RequestError {
    TCPErr(TCPError),
    RequestSerializationError,
    ResponseDeserializationError,
    EmptyResponse,
    ErrorResponse,
    InvalidResponse,
}
