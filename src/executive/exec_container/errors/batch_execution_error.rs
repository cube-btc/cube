use crate::executive::exec_container::errors::liftup_execution_error::LiftupExecutionError;
use crate::constructive::entries::ape::decode::error::decode_error::EntryAPEDecodeError;

/// Errors associated with executing a batch of entries.
#[derive(Debug, Clone)]
pub enum BatchExecutionError {
    DecodeEntryError(EntryAPEDecodeError),
    LiftupExecutionError(LiftupExecutionError),
}
