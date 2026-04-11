use crate::constructive::core_types::valtypes::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::core_types::valtypes::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;
use crate::constructive::entry::entry::ext::codec::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;
use crate::constructive::entries::entry_kinds::liftup::ext::signature::sighash::error::sighash_error::LiftupSighashError;

/// Errors associated with executing a batch of entries.
#[derive(Debug, Clone)]
pub enum BatchExecutionError {
    BatchTemplatePayloadBitsConversionError(Vec<u8>),
    DecodePayloadVersionError(ShortValAPEDecodeError),
    DecodeBatchTimestampError(LongValAPEDecodeError),
    DecodeAggregateBLSSignatureError,
    DecodeEntryError(EntryAPEDecodeError),
    LiftupExecutionError(LiftupExecutionError),
    LiftupSighashError(LiftupSighashError),
    AggregateBLSSignatureVerificationError,
}
