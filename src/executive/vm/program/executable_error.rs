use std::fmt;

/// A section of executable block in the `Contract`.    
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableConstructionError {
    /// Executable name length error.
    ExecutableNameLengthError,
    /// Method count error.
    MethodCountError,
    /// Method validation error.
    MethodValidationError(MethodValidationError),
}

impl fmt::Display for ExecutableConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutableConstructionError::ExecutableNameLengthError => {
                write!(f, "Executable name length is invalid")
            }
            ExecutableConstructionError::MethodCountError => {
                write!(f, "Invalid method count")
            }
            ExecutableConstructionError::MethodValidationError(err) => {
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
