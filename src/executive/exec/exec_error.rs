use crate::executive::stack::{stack_error::StackError, stack_item::StackItem};
use std::fmt;

/// A section of executable block in the `Contract`.    
#[derive(Debug, Clone)]
pub enum ExecutionError {
    /// Program not found error.
    ProgramNotFoundError([u8; 32]),
    /// Method not found at index error.
    MethodNotFoundAtIndexError(u8),
    /// Read only call error.
    ReadOnlyCallEncounteredError,
    /// Internal call caller is not the contract error.
    InvalidInternalCallError,
    /// Invalid callable call error.
    InvalidCallableCallError,
    /// Stack holder initialization error.
    StackHolderInitializationError(StackError),
    /// Opcode execution error.
    OpcodeExecutionError(StackError),
    /// Method not returned any items error.
    MethodNotReturnedAnyItemsError,
    /// Invalid external call attempt as internal call error.
    ExternalCallAttemptAsInternalError,
    /// Return error item error.
    ReturnErrorFromStackError(StackItem),
    /// Arg type mismatch error.
    ArgTypeMismatchError,
    /// Min payable allocation error.
    MinPayableAllocationError,
    /// Payable allocation caller is not an account error.
    PayableAllocationCallerIsNotAnAccountError,
    /// Payable with internal call error.
    PayableWithInternalCallError,
    /// Invalid stack ending error.
    InvalidStackEndingError,
    /// Base ops price mismatch error.
    BaseOpsPriceMismatchError,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::ProgramNotFoundError(contract_id) => {
                write!(f, "Program not found at contract id: {:?}", contract_id)
            }
            ExecutionError::MethodNotFoundAtIndexError(index) => {
                write!(f, "Method not found at index: {}", index)
            }
            ExecutionError::ReadOnlyCallEncounteredError => {
                write!(f, "Read only call encountered")
            }
            ExecutionError::InvalidInternalCallError => {
                write!(f, "Invalid internal call")
            }
            ExecutionError::InvalidCallableCallError => {
                write!(f, "Invalid callable call")
            }
            ExecutionError::StackHolderInitializationError(error) => {
                write!(f, "Stack holder initialization error: {:?}", error)
            }
            ExecutionError::OpcodeExecutionError(error) => {
                write!(f, "Opcode execution error: {:?}", error)
            }
            ExecutionError::MethodNotReturnedAnyItemsError => {
                write!(f, "Method not returned any items")
            }
            ExecutionError::ExternalCallAttemptAsInternalError => {
                write!(f, "External call attempt as internal call")
            }
            ExecutionError::ReturnErrorFromStackError(error) => {
                write!(f, "Return error from stack: {:?}", error)
            }
            ExecutionError::ArgTypeMismatchError => {
                write!(f, "Arg type mismatch")
            }
            ExecutionError::MinPayableAllocationError => {
                write!(f, "Min payable allocation error")
            }
            ExecutionError::PayableAllocationCallerIsNotAnAccountError => {
                write!(f, "Payable allocation caller is not an account")
            }
            ExecutionError::PayableWithInternalCallError => {
                write!(f, "Payable with internal call")
            }
            ExecutionError::InvalidStackEndingError => {
                write!(f, "Invalid stack ending")
            }
            ExecutionError::BaseOpsPriceMismatchError => {
                write!(f, "Base ops price mismatch")
            }
        }
    }
}
