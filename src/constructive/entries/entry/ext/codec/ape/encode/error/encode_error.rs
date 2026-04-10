use crate::constructive::entry::entry_types::call::ape::encode::error::encode_error::CallAPEEncodeError;
use crate::constructive::entry::entry_types::liftup::ext::codec::ape::encode::error::encode_error::LiftupAPEEncodeError;

/// Enum to represent errors that can occur when encoding an `Entry` as an Airly Payload Encoding (APE) bit vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryAPEEncodeError {
    CallAPEEncodeError(CallAPEEncodeError),
    LiftupAPEEncodeError(LiftupAPEEncodeError),
}
