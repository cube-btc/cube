use crate::constructive::entries::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_ctx::errors::apply_changes_error::ApplyChangesError;

/// Errors associated with executing a batch of entries.
#[derive(Debug, Clone)]
pub enum BatchExecutionError {
    BatchTemplatePayloadBitsConversionError(Vec<u8>),
    DecodeEntryError(EntryAPEDecodeError),
    LiftupExecutionError(LiftupExecutionError),
    ApplyChangesError(ApplyChangesError),
}
