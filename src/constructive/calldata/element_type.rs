use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the type of a single element of calldata.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalldataElementType {
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

impl CalldataElementType {
    /// Returns the bytecode of the element type.
    pub fn bytecode(&self) -> Vec<u8> {
        match self {
            CalldataElementType::U8 => vec![0x00],
            CalldataElementType::U16 => vec![0x01],
            CalldataElementType::U32 => vec![0x02],
            CalldataElementType::U64 => vec![0x03],
            CalldataElementType::Bool => vec![0x04],
            CalldataElementType::Account => vec![0x05],
            CalldataElementType::Contract => vec![0x06],
            CalldataElementType::Bytes(index) => {
                // Return the bytes.
                vec![0x07, index.to_owned()]
            }
            CalldataElementType::Varbytes => vec![0x08],
            CalldataElementType::Payable => vec![0x09],
        }
    }

    /// Returns the element type from the bytecode.
    pub fn from_bytecode<I>(bytecode_stream: &mut I) -> Option<Self>
    where
        I: Iterator<Item = u8>,
    {
        match bytecode_stream.next() {
            Some(0x00) => Some(CalldataElementType::U8),
            Some(0x01) => Some(CalldataElementType::U16),
            Some(0x02) => Some(CalldataElementType::U32),
            Some(0x03) => Some(CalldataElementType::U64),
            Some(0x04) => Some(CalldataElementType::Bool),
            Some(0x05) => Some(CalldataElementType::Account),
            Some(0x06) => Some(CalldataElementType::Contract),
            Some(0x07) => bytecode_stream
                .next()
                .map(|index| CalldataElementType::Bytes(index)),
            Some(0x08) => Some(CalldataElementType::Varbytes),
            Some(0x09) => Some(CalldataElementType::Payable),
            _ => None,
        }
    }
}

impl fmt::Display for CalldataElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalldataElementType::U8 => write!(f, "U8"),
            CalldataElementType::U16 => write!(f, "U16"),
            CalldataElementType::U32 => write!(f, "U32"),
            CalldataElementType::U64 => write!(f, "U64"),
            CalldataElementType::Bool => write!(f, "Bool"),
            CalldataElementType::Account => write!(f, "Account"),
            CalldataElementType::Contract => write!(f, "Contract"),
            CalldataElementType::Bytes(index) => write!(f, "Bytes{}", (index + 1)),
            CalldataElementType::Varbytes => write!(f, "Varbytes"),
            CalldataElementType::Payable => write!(f, "Payable"),
        }
    }
}
