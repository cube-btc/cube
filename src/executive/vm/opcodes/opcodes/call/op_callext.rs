use crate::executive::stack::{
    stack_error::{CallError, StackError},
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::{SafeConverter, StackItemUintExt},
};
use serde::{Deserialize, Serialize};

/// Calls an external contract method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_CALLEXT;

/// The number of ops for the `OP_CALLEXT` opcode.
pub const CALLEXT_OPS: u32 = 50;

/// Contract id to be called.
type ContractIdToBeCalled = [u8; 32];
/// Method index to be called.
type MethodIndexToBeCalled = u8;
/// Call arguments.
type CallArguments = Vec<StackItem>;

/// The `OP_CALLEXT` opcode.
impl OP_CALLEXT {
    /// Execute the `OP_CALLEXT` opcode.
    pub fn execute(
        stack_holder: &mut StackHolder,
    ) -> Result<(ContractIdToBeCalled, MethodIndexToBeCalled, CallArguments), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(([0xff; 32], 0xff, vec![]));
        }

        // Pop the contract id from the stack.
        let contract_id = stack_holder.pop()?;

        // Pop the method index from the stack.
        let method_index = stack_holder.pop()?;

        // Pop the number of arguments from the stack.
        let arguments_count = stack_holder.pop()?;

        // Convert the args count to a u32.
        let args_count_as_u32 = match arguments_count.to_stack_uint() {
            Some(value) => match value.to_u32() {
                Some(u32_value) => u32_value,
                None => return Err(StackError::CallError(CallError::InvalidArgumentsCount)),
            },
            None => return Err(StackError::CallError(CallError::InvalidArgumentsCount)),
        };

        // Convert the contract id and method index to bytes.
        let contract_id_bytes: [u8; 32] = match contract_id.bytes().try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(StackError::CallError(CallError::InvalidContractId)),
        };

        // Convert the method index to a u32.
        let method_index_as_u32: u32 = match method_index.to_stack_uint() {
            Some(value) => match value.to_u32() {
                Some(u32_value) => u32_value,
                None => return Err(StackError::CallError(CallError::InvalidMethodIndex)),
            },
            None => return Err(StackError::CallError(CallError::InvalidMethodIndex)),
        };

        // Convert the method index to a u8.
        let method_index_as_u8: u8 = match method_index_as_u32 {
            u32_value if u32_value > u8::MAX as u32 => {
                return Err(StackError::CallError(CallError::InvalidMethodIndex))
            }
            u32_value => u32_value as u8,
        };

        // Initialize a vector to store the arguments.
        let mut arguments = Vec::<StackItem>::with_capacity(args_count_as_u32 as usize);

        // Collect remaining stack items.
        for _ in 0..args_count_as_u32 {
            arguments.push(stack_holder.pop()?);
        }

        // Increment the ops counter.
        stack_holder.increment_ops(CALLEXT_OPS)?;

        Ok((contract_id_bytes, method_index_as_u8, arguments))
    }

    /// Returns the bytecode for the `OP_CALLEXT` opcode (0xbf).
    pub fn bytecode() -> Vec<u8> {
        vec![0xbf]
    }
}
