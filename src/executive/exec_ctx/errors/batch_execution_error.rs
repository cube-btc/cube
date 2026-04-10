use crate::constructive::core_types::valtypes::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::core_types::valtypes::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;
use crate::constructive::entry::entry::ext::codec::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;

/// Errors associated with executing a batch of entries.
#[derive(Debug, Clone)]
pub enum BatchExecutionError {
    BatchTemplatePayloadBitsConversionError(Vec<u8>),
    DecodePayloadVersionError(ShortValAPEDecodeError),
    DecodeBatchTimestampError(LongValAPEDecodeError),
    DecodeEntryError(EntryAPEDecodeError),
    LiftupExecutionError(LiftupExecutionError),
}
