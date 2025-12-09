use crate::executive::stack::{
    stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem,
};
use bitcoin::hashes::sha1;
use bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};

/// The input is hashed using SHA-1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SHA1;

/// The number of ops for the `OP_SHA1` opcode.
pub const SHA1_OPS: u32 = 30;

impl OP_SHA1 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the preimage from the main stack.
        let preimage = stack_holder.pop()?;

        // Hash the item using SHA-1.
        let hash = sha1::Hash::hash(preimage.bytes()).to_byte_array().to_vec();

        // Increment the ops counter.
        stack_holder.increment_ops(SHA1_OPS)?;

        // Push the hash back to the main stack.
        stack_holder.push(StackItem::new(hash))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SHA1` opcode (0xa7).
    pub fn bytecode() -> Vec<u8> {
        vec![0xa7]
    }
}
