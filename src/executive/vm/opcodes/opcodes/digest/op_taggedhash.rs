use crate::executive::stack::{
    stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem,
};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use crate::transmutative::hash::{Hash, HashTag};
use serde::{Deserialize, Serialize};

/// The input is hashed with a domain separation tag.
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
        stack_holder.increment_ops(calculate_ops(preimage.len(), stack_holder))?;

        // Push the hash back to the main stack.
        stack_holder.push(StackItem::new(hash.to_vec()))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_TAGGEDHASH` opcode (0xab).
    pub fn bytecode() -> Vec<u8> {
        vec![0xab]
    }
}

// Calculate the number of ops for a OP_TAGGEDHASH opcode.
fn calculate_ops(preimage_len: u32, stack_holder: &StackHolder) -> u32 {
    let ops = stack_holder.opcode_ops();
    let output_len = OpcodeOpsParams::as_u32(ops.op_taggedhash_output_len);
    let gap = output_len.saturating_sub(preimage_len);

    OpcodeOpsParams::as_u32(ops.op_taggedhash_base)
        + (OpcodeOpsParams::as_u32(ops.op_taggedhash_per_byte_gap) * gap)
}
