use crate::constructive::entry::entries::call::ape::encode::error::encode_error::CallAPEEncodeError;

/// Enum to represent errors that can occur when encoding an `Entry` as an Airly Payload Encoding (APE) bit vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryAPEEncodeError {
    CallAPEEncodeError(CallAPEEncodeError),
}
