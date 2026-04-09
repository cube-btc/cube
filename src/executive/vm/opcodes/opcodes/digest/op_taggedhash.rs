use crate::executive::stack::{
    stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem,
};
use crate::transmutative::hash::{Hash, HashTag};
use serde::{Deserialize, Serialize};

/// The input is hashed with a domain seperation tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_TAGGEDHASH;

impl OP_TAGGEDHASH {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the tag from the main stack.
        let tag = stack_holder.pop()?;

        // Pop the preimage from the main stack.
        let preimage = stack_holder.pop()?;

        // Hash the item with the given tag.
        let hash = match tag.is_true() {
            // The tag is non-empty.
            true => preimage
                .bytes()
                .hash(Some(HashTag::CustomBytes(tag.bytes().to_vec()))),
            // The tag is empty.
            false => preimage.bytes().hash(None),
        };

        // Increment the ops counter.
        stack_holder.increment_ops(calculate_ops(preimage.len()))?;

        // Push the hash back to the main stack.
        stack_holder.push(StackItem::new(hash.to_vec()))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_TAGGEDHASH` opcode (0xab).
    pub fn bytecode() -> Vec<u8> {
        vec![0xab]
    }
}

const TAGGEDHASH_OPS_BASE: u32 = 10;
const TAGGEDHASH_OPS_MULTIPLIER: u32 = 1;
const TAGGEDHASH_OPS_OUTPUT_LEN: u32 = 32;

// Calculate the number of ops for a OP_TAGGEDHASH opcode.
fn calculate_ops(preimage_len: u32) -> u32 {
    // Calculate the gap between the preimage length and the output length.
    let gap = match TAGGEDHASH_OPS_OUTPUT_LEN.checked_sub(preimage_len) {
        Some(gap) => gap,
        None => 0,
    };

    // Return the number of ops.
    TAGGEDHASH_OPS_BASE + (TAGGEDHASH_OPS_MULTIPLIER * gap)
}
