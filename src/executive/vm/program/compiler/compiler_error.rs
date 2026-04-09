use crate::executive::executable::executable_error::ExecutableConstructionError;
use crate::executive::executable::method::compiler::compiler_error::{
    MethodCompileError, MethodDecompileError,
};
use std::fmt;

/// The error that occurs when compiling a program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableCompileError {
    /// The method compile error.
    MethodCompileError(MethodCompileError),
}

impl fmt::Display for ExecutableCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutableCompileError::MethodCompileError(err) => {
                write!(f, "Method compile error: {}", err)
            }
        }
    }
}

/// The error that occurs when decompiling a executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableDecompileError {
    /// The program name length byte collect error.
    NameLengthByteCollectError,
    /// The executable name bytes collect error.
    ExecutableNameBytesCollectError,
    /// The deployed by bytes collect error.
    DeployedByBytesCollectError,
    /// The method count byte collect error.
    MethodCountByteCollectError,
    /// The method decompile error.
    MethodDecompileError(MethodDecompileError),
    /// The executable construct error.
    ExecutableConstructError(ExecutableConstructionError),
}

impl fmt::Display for ExecutableDecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutableDecompileError::NameLengthByteCollectError => {
                write!(f, "Failed to collect executable name length byte")
            }
            ExecutableDecompileError::ExecutableNameBytesCollectError => {
                write!(f, "Failed to collect executable name bytes")
            }
            ExecutableDecompileError::DeployedByBytesCollectError => {
                write!(f, "Failed to collect deployed by bytes")
            }
            ExecutableDecompileError::MethodCountByteCollectError => {
                write!(f, "Failed to collect method count byte")
            }
            ExecutableDecompileError::MethodDecompileError(err) => {
                write!(f, "Method decompile error: {}", err)
            }
            ExecutableDecompileError::ExecutableConstructError(err) => {
                write!(f, "Executable construction error: {}", err)
            }
        }
    }
}
