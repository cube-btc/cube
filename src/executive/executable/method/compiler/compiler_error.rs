use crate::executive::executable::method::method_error::MethodConstructionError;
use crate::executive::opcode::compiler::compiler_error::{
    OpcodeCompileError, OpcodeDecompileError,
};
use std::fmt;

/// The error that occurs when compiling a method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodCompileError {
    /// The opcode compile error.
    OpcodeCompileError(OpcodeCompileError),
}

impl fmt::Display for MethodCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MethodCompileError::OpcodeCompileError(err) => {
                write!(f, "Opcode compile error: {}", err)
            }
        }
    }
}

/// The error that occurs when decompiling a method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodDecompileError {
    /// The method name length byte collect error.
    NameLengthByteCollectError,
    /// The method name bytes collect error.
    NameBytesCollectError,
    /// The method type byte collect error.
    MethodTypeByteCollectError,
    /// The method type is invalid.
    InvalidMethodType,
    /// The call element type count byte collect error.
    NumberOfCallElementTypesByteCollectError,
    /// The call element type is invalid.
    InvalidCallElementType,
    /// The opcode decompile error.
    OpcodeDecompileError(OpcodeDecompileError),
    /// The method construct error.
    MethodConstructError(MethodConstructionError),
}

impl fmt::Display for MethodDecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MethodDecompileError::NameLengthByteCollectError => {
                write!(f, "Failed to collect method name length byte")
            }
            MethodDecompileError::NameBytesCollectError => {
                write!(f, "Failed to collect method name bytes")
            }
            MethodDecompileError::MethodTypeByteCollectError => {
                write!(f, "Failed to collect method type byte")
            }
            MethodDecompileError::InvalidMethodType => {
                write!(f, "Invalid method type")
            }
            MethodDecompileError::NumberOfCallElementTypesByteCollectError => {
                write!(f, "Failed to collect number of call element types byte")
            }
            MethodDecompileError::InvalidCallElementType => {
                write!(f, "Invalid call element type")
            }
            MethodDecompileError::OpcodeDecompileError(err) => {
                write!(f, "Opcode decompile error: {}", err)
            }
            MethodDecompileError::MethodConstructError(err) => {
                write!(f, "Method construction error: {}", err)
            }
        }
    }
}
