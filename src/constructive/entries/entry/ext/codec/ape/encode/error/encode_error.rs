use crate::constructive::entry::entry_kinds::call::ape::encode::error::encode_error::CallAPEEncodeError;
use crate::constructive::entry::entry_kinds::liftup::ext::codec::ape::encode::error::encode_error::LiftupAPEEncodeError;
use crate::constructive::entry::entry_kinds::r#move::ext::codec::ape::encode::error::encode_error::MoveAPEEncodeError;
use crate::constructive::entry::entry_kinds::swapout::ext::codec::ape::encode::error::encode_error::SwapoutAPEEncodeError;

/// Enum to represent errors that can occur when encoding an `Entry` as an Airly Payload Encoding (APE) bit vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryAPEEncodeError {
    MoveAPEEncodeError(MoveAPEEncodeError),
    CallAPEEncodeError(CallAPEEncodeError),
    LiftupAPEEncodeError(LiftupAPEEncodeError),
    SwapoutAPEEncodeError(SwapoutAPEEncodeError),
}
