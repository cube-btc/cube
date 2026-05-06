use crate::executive::executable::program_error::ProgramConstructionError;
use crate::executive::executable::method::compiler::compiler_error::{
    MethodCompileError, MethodDecompileError,
};
use std::fmt;

/// The error that occurs when compiling a program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramCompileError {
    /// The method compile error.
    MethodCompileError(MethodCompileError),
}

impl fmt::Display for ProgramCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramCompileError::MethodCompileError(err) => {
                write!(f, "Method compile error: {}", err)
            }
        }
    }
}

/// The error that occurs when decompiling a executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramDecompileError {
    /// The program name length byte collect error.
    NameLengthByteCollectError,
    /// The program name bytes collect error.
    ProgramNameBytesCollectError,
    /// The metadata presence flag byte collect error.
    MetadataFlagByteCollectError,
    /// The metadata length bytes collect error.
    MetadataLengthBytesCollectError,
    /// The metadata bytes collect error.
    MetadataBytesCollectError,
    /// The method count byte collect error.
    MethodCountByteCollectError,
    /// The method decompile error.
    MethodDecompileError(MethodDecompileError),
    /// The program construct error.
    ProgramConstructError(ProgramConstructionError),
}

impl fmt::Display for ProgramDecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramDecompileError::NameLengthByteCollectError => {
                write!(f, "Failed to collect program name length byte")
            }
            ProgramDecompileError::ProgramNameBytesCollectError => {
                write!(f, "Failed to collect program name bytes")
            }
            ProgramDecompileError::MetadataFlagByteCollectError => {
                write!(f, "Failed to collect metadata flag byte")
            }
            ProgramDecompileError::MetadataLengthBytesCollectError => {
                write!(f, "Failed to collect metadata length bytes")
            }
            ProgramDecompileError::MetadataBytesCollectError => {
                write!(f, "Failed to collect metadata bytes")
            }
            ProgramDecompileError::MethodCountByteCollectError => {
                write!(f, "Failed to collect method count byte")
            }
            ProgramDecompileError::MethodDecompileError(err) => {
                write!(f, "Method decompile error: {}", err)
            }
            ProgramDecompileError::ProgramConstructError(err) => {
                write!(f, "Program construction error: {}", err)
            }
        }
    }
}

pub type ExecutableCompileError = ProgramCompileError;
pub type ExecutableDecompileError = ProgramDecompileError;
