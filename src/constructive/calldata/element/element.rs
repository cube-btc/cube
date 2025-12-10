use crate::constructive::calldata::element_type::CalldataElementType;
use crate::constructive::entity::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::executive::stack::stack_item::StackItem;
use crate::executive::stack::stack_uint::{SafeConverter, StackItemUintExt, StackUint};
use serde::{Deserialize, Serialize};

// Represents a single element of calldata.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CalldataElement {
    U8(u8),
    U16(u16),
    U32(ShortVal),
    U64(LongVal),
    Bool(bool),
    Account(Account),
    Contract(Contract),
    Bytes(Vec<u8>),
    Varbytes(Vec<u8>),
    Payable(ShortVal),
}

impl CalldataElement {
    /// Returns the type of the element.
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
                let value_as_stack_uint = StackUint::from_u32(value.value());

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
            // 0-8 bytes in stack.
            CalldataElement::U64(value) => {
                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u64(value.value());

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
            // 0-4096 bytes in stack.
            CalldataElement::Varbytes(bytes) => StackItem::new(bytes.clone()),
            // 0-4 bytes in stack.
            CalldataElement::Payable(value) => {
                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u32(value.value());

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
        }
    }
}
