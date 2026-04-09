use crate::constructive::entries::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;

/// Errors associated with executing a batch of entries.
#[derive(Debug, Clone)]
pub enum BatchExecutionError {
    DecodeEntryError(EntryAPEDecodeError),
    LiftupExecutionError(LiftupExecutionError),
}
