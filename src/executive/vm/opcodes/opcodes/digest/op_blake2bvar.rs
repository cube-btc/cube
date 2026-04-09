use crate::executive::stack::stack_uint::StackItemUintExt;
use crate::executive::stack::{
    stack_error::{StackError, StackUintError},
    stack_holder::StackHolder,
    stack_item::StackItem,
};
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use serde::{Deserialize, Serialize};

// The maximum output size for the BLAKE2b variable output.
const BLAKE2B_VAR_MAX_OUTPUT_SIZE: usize = 64;

/// The input is hashed with a variable length output using BLAKE2b.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_BLAKE2BVAR;

impl OP_BLAKE2BVAR {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the ouput size from the main stack.
        let output_size = stack_holder.pop()?;

        // Pop the preimage from the main stack.
        let preimage = stack_holder.pop()?;

        // Convert the output size to a u32.
        let output_size_as_usize = output_size
            .to_stack_uint()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ))?
            .as_usize();

        // Check if the output size is valid.
        if output_size_as_usize > BLAKE2B_VAR_MAX_OUTPUT_SIZE {
            return Err(StackError::BLAKE2bVarOutputSizeError);
        }

        // Create a new BLAKE2b hasher with the given output size.
        let mut hasher = Blake2bVar::new(output_size_as_usize)
            .map_err(|_| StackError::BLAKE2bVarOutputSizeError)?;

        // Update the hasher with the preimage.
        hasher.update(&preimage.bytes());

        // Create a buffer to store the hash.
        let mut output_buffer = vec![0u8; output_size_as_usize];

        hasher.finalize_variable(&mut output_buffer).unwrap();

        // Increment the ops counter.
        stack_holder.increment_ops(calculate_ops(output_size_as_usize as u32))?;

        // Push the hash back to the main stack.
        stack_holder.push(StackItem::new(output_buffer))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_BLAKE2BVAR` opcode (0xac).
    pub fn bytecode() -> Vec<u8> {
        vec![0xac]
    }
}

const BLAKE2BVAR_OPS_BASE: u32 = 10;
const BLAKE2BVAR_OPS_MULTIPLIER: u32 = 1;

// Calculate the number of ops for a OP_BLAKE2BVAR opcode.
fn calculate_ops(output_size: u32) -> u32 {
    // Return the number of ops.
    BLAKE2BVAR_OPS_BASE + (BLAKE2BVAR_OPS_MULTIPLIER * output_size)
}
