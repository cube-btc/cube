use crate::constructive::core_types::calldata::calldata_elements::validation::{
    CalldataElementValidationError, MAX_BYTES_LEN, MAX_VARBYTES_LEN, MIN_BYTES_LEN,
};
use crate::constructive::core_types::calldata::element_type::CalldataElementType;
use crate::constructive::core_types::entities::account::account::account::Account;
use crate::constructive::core_types::entities::contract::contract::Contract;
use crate::executive::stack::stack_item::StackItem;
use crate::executive::stack::stack_uint::{SafeConverter, StackItemUintExt, StackUint};
use serde::{Deserialize, Serialize};

// Represents a single element of calldata.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CalldataElement {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Bool(bool),
    Account(Account),
    Contract(Contract),
    Bytes(Vec<u8>),
    Varbytes(Vec<u8>),
    Payable(u32),
}

impl CalldataElement {
    /// Checks payload invariants for [`Bytes`] and [`Varbytes`] variants.
    pub fn validate(&self) -> Result<(), CalldataElementValidationError> {
        match self {
            CalldataElement::Bytes(bytes) => {
                if bytes.len() < MIN_BYTES_LEN {
                    return Err(CalldataElementValidationError::EmptyBytes);
                }
                if bytes.len() > MAX_BYTES_LEN {
                    return Err(CalldataElementValidationError::BytesLengthOutOfRange {
                        len: bytes.len(),
                    });
                }
            }
            CalldataElement::Varbytes(bytes) => {
                if bytes.len() > MAX_VARBYTES_LEN {
                    return Err(CalldataElementValidationError::VarbytesLengthExceedsMax {
                        len: bytes.len(),
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Returns the type of the element.
    ///
    /// For [`CalldataElement::Bytes`], the vector must satisfy [`Self::validate`] (length 1–256).
    pub fn element_type(&self) -> CalldataElementType {
        // Match on the element type.
        match self {
            CalldataElement::U8(_) => CalldataElementType::U8,
            CalldataElement::U16(_) => CalldataElementType::U16,
            CalldataElement::U32(_) => CalldataElementType::U32,
            CalldataElement::U64(_) => CalldataElementType::U64,
            CalldataElement::Bool(_) => CalldataElementType::Bool,
            CalldataElement::Account(_) => CalldataElementType::Account,
            CalldataElement::Contract(_) => CalldataElementType::Contract,
            CalldataElement::Bytes(bytes) => {
                // Byte length is the inner value + 1. So we need to subtract 1 from the length.
                let index = bytes.len() as u8 - 1;
                // Return the element type.
                CalldataElementType::Bytes(index)
            }
            CalldataElement::Varbytes(_) => CalldataElementType::Varbytes,
            CalldataElement::Payable(_) => CalldataElementType::Payable,
        }
    }

    /// Returns the element in the pure bytes format to be pushed/used for stack operations.
    pub fn into_stack_item(&self) -> StackItem {
        match self {
            // 0-1 bytes in stack.
            CalldataElement::U8(value) => {
                // Convert the value to a u32.
                let value_as_u32 = *value as u32;

                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u32(value_as_u32);

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
            // 0-2 bytes in stack.
            CalldataElement::U16(value) => {
                // Convert the value to a u32.
                let value_as_u32 = *value as u32;

                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u32(value_as_u32);

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
            // 0-4 bytes in stack.
            CalldataElement::U32(value) => {
                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u32(*value);

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
            // 0-8 bytes in stack.
            CalldataElement::U64(value) => {
                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u64(*value);

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
            // 0-1 bytes in stack.
            CalldataElement::Bool(value) => match value {
                // True is a single byte of 0x01.
                true => StackItem::true_item(),
                // False is an empty stack item.
                false => StackItem::false_item(),
            },
            // 32 bytes in stack.
            CalldataElement::Account(value) => StackItem::new(value.account_key().to_vec()),
            // 32 bytes in stack.
            CalldataElement::Contract(value) => StackItem::new(value.contract_id().to_vec()),
            // 1-256 bytes in stack.
            CalldataElement::Bytes(bytes) => StackItem::new(bytes.clone()),
            // 0-4095 bytes in stack.
            CalldataElement::Varbytes(bytes) => StackItem::new(bytes.clone()),
            // 0-4 bytes in stack.
            CalldataElement::Payable(value) => {
                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u32(*value);

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
        }
    }
}
