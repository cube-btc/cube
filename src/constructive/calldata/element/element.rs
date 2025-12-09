use crate::constructive::calldata::element_type::CallElementType;
use crate::constructive::entity::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::executive::stack::stack_item::StackItem;
use crate::executive::stack::stack_uint::{SafeConverter, StackItemUintExt, StackUint};
use serde::{Deserialize, Serialize};

// Represents a single element of calldata.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CallElement {
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

impl CallElement {
    /// Returns the type of the element.
    pub fn element_type(&self) -> CallElementType {
        // Match on the element type.
        match self {
            CallElement::U8(_) => CallElementType::U8,
            CallElement::U16(_) => CallElementType::U16,
            CallElement::U32(_) => CallElementType::U32,
            CallElement::U64(_) => CallElementType::U64,
            CallElement::Bool(_) => CallElementType::Bool,
            CallElement::Account(_) => CallElementType::Account,
            CallElement::Contract(_) => CallElementType::Contract,
            CallElement::Bytes(bytes) => {
                // Byte length is the inner value + 1. So we need to subtract 1 from the length.
                let index = bytes.len() as u8 - 1;
                // Return the element type.
                CallElementType::Bytes(index)
            }
            CallElement::Varbytes(_) => CallElementType::Varbytes,
            CallElement::Payable(_) => CallElementType::Payable,
        }
    }

    /// Returns the element in the pure bytes format to be pushed/used for stack operations.
    pub fn into_stack_item(&self) -> StackItem {
        match self {
            // 0-1 bytes in stack.
            CallElement::U8(value) => {
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
            CallElement::U16(value) => {
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
            CallElement::U32(value) => {
                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u32(value.value());

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
            // 0-8 bytes in stack.
            CallElement::U64(value) => {
                // Convert the value to a `StackUint`.
                let value_as_stack_uint = StackUint::from_u64(value.value());

                // Convert the value to a `StackItem`.
                let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                // Return the stack item.
                value_as_stack_item
            }
            // 0-1 bytes in stack.
            CallElement::Bool(value) => match value {
                // True is a single byte of 0x01.
                true => StackItem::true_item(),
                // False is an empty stack item.
                false => StackItem::false_item(),
            },
            // 32 bytes in stack.
            CallElement::Account(value) => StackItem::new(value.account_key().to_vec()),
            // 32 bytes in stack.
            CallElement::Contract(value) => StackItem::new(value.contract_id().to_vec()),
            // 1-256 bytes in stack.
            CallElement::Bytes(bytes) => StackItem::new(bytes.clone()),
            // 0-4096 bytes in stack.
            CallElement::Varbytes(bytes) => StackItem::new(bytes.clone()),
            // 0-4 bytes in stack.
            CallElement::Payable(value) => {
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
