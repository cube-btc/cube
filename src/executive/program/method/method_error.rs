use std::fmt;

/// A section of executable block in the `Contract`.    
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodConstructionError {
    /// Method name length error.
    MethodNameLengthError,
    /// Arg count error.
    ArgCountError,
    /// Arg validation error.
    ArgValidationError,
    /// Opcode count error.
    OpcodeCountError,
    /// Script validation error.
    ScriptValidationError(ScriptValidationError),
}

impl fmt::Display for MethodConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MethodConstructionError::MethodNameLengthError => {
                write!(f, "Method name length is invalid")
            }
            MethodConstructionError::ArgCountError => {
                write!(f, "Invalid arg count")
            }
            MethodConstructionError::OpcodeCountError => {
                write!(f, "Invalid opcode count")
            }
            MethodConstructionError::ScriptValidationError(err) => {
                write!(f, "Script validation error: {}", err)
            }
            MethodConstructionError::ArgValidationError => {
                write!(f, "Arg validation error")
            }
        }
    }
}

/// A section of executable block in the `Contract`.    
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptValidationError {
    /// Non minimal data push error.
    NonMinimalDataPushError,
}

impl fmt::Display for ScriptValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptValidationError::NonMinimalDataPushError => {
                write!(f, "Non-minimal data push encountered")
            }
        }
    }
}
