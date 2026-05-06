use std::fmt;

/// A section of program block in the `Contract`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramConstructionError {
    /// Program name length error.
    ProgramNameLengthError,
    /// Method count error.
    MethodCountError,
    /// Method validation error.
    MethodValidationError(MethodValidationError),
}

impl fmt::Display for ProgramConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramConstructionError::ProgramNameLengthError => {
                write!(f, "Program name length is invalid")
            }
            ProgramConstructionError::MethodCountError => {
                write!(f, "Invalid method count")
            }
            ProgramConstructionError::MethodValidationError(err) => {
                write!(f, "Method validation error: {}", err)
            }
        }
    }
}

/// The error that occurs when validating the methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodValidationError {
    /// Duplicate method name error.
    DuplicateMethodNameError,
    /// All method types are internal error.
    AllMethodTypesAreInternal,
}

impl fmt::Display for MethodValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MethodValidationError::DuplicateMethodNameError => {
                write!(f, "Duplicate method name found")
            }
            MethodValidationError::AllMethodTypesAreInternal => {
                write!(f, "All method types are internal")
            }
        }
    }
}

pub type ExecutableConstructionError = ProgramConstructionError;
