use crate::constructive::entry::entries::call::ape::decode::error::decode_error::CallEntryAPEDecodeError;

/// Enum to represent errors that can occur when decoding an `Entry` from an Airly Payload Encoding (APE) bitstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryAPEDecodeError {
    CommonUncommonBranchBitCollectError,
    MoveOrCallBitCollectError,
    CallEntryAPEDecodeError(CallEntryAPEDecodeError),
}
