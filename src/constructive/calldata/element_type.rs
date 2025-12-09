use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the type of a single element of calldata.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallElementType {
    // Represents an unsigned 8-bit integer.
    U8,
    // Represents an unsigned 16-bit integer.
    U16,
    // Represents an unsigned 32-bit integer.
    U32,
    // Represents an unsigned 64-bit integer.
    U64,
    // Represents a boolean value.
    Bool,
    // Represents an `Account`.
    Account,
    // Represents a `Contract`.
    Contract,
    // Represents a byte array with a known length.
    // Byte length is the inner value + 1.
    // Supported byte-length range: 1-256 bytes
    Bytes(u8),
    // Represents a byte array with an unknown length.
    // Supported byte-length range: 0-4096 bytes
    Varbytes,
    // Represents a payable value.
    Payable,
}

impl CallElementType {
    /// Returns the bytecode of the element type.
    pub fn bytecode(&self) -> Vec<u8> {
        match self {
            CallElementType::U8 => vec![0x00],
            CallElementType::U16 => vec![0x01],
            CallElementType::U32 => vec![0x02],
            CallElementType::U64 => vec![0x03],
            CallElementType::Bool => vec![0x04],
            CallElementType::Account => vec![0x05],
            CallElementType::Contract => vec![0x06],
            CallElementType::Bytes(index) => {
                // Return the bytes.
                vec![0x07, index.to_owned()]
            }
            CallElementType::Varbytes => vec![0x08],
            CallElementType::Payable => vec![0x09],
        }
    }

    /// Returns the element type from the bytecode.
    pub fn from_bytecode<I>(bytecode_stream: &mut I) -> Option<Self>
    where
        I: Iterator<Item = u8>,
    {
        match bytecode_stream.next() {
            Some(0x00) => Some(CallElementType::U8),
            Some(0x01) => Some(CallElementType::U16),
            Some(0x02) => Some(CallElementType::U32),
            Some(0x03) => Some(CallElementType::U64),
            Some(0x04) => Some(CallElementType::Bool),
            Some(0x05) => Some(CallElementType::Account),
            Some(0x06) => Some(CallElementType::Contract),
            Some(0x07) => bytecode_stream
                .next()
                .map(|index| CallElementType::Bytes(index)),
            Some(0x08) => Some(CallElementType::Varbytes),
            Some(0x09) => Some(CallElementType::Payable),
            _ => None,
        }
    }
}

impl fmt::Display for CallElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallElementType::U8 => write!(f, "U8"),
            CallElementType::U16 => write!(f, "U16"),
            CallElementType::U32 => write!(f, "U32"),
            CallElementType::U64 => write!(f, "U64"),
            CallElementType::Bool => write!(f, "Bool"),
            CallElementType::Account => write!(f, "Account"),
            CallElementType::Contract => write!(f, "Contract"),
            CallElementType::Bytes(index) => write!(f, "Bytes{}", (index + 1)),
            CallElementType::Varbytes => write!(f, "Varbytes"),
            CallElementType::Payable => write!(f, "Payable"),
        }
    }
}
