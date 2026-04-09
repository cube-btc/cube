use crate::executive::{
    opcode::ops::OP_RSHIFT_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Shifts a right b bits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_RSHIFT;

impl OP_RSHIFT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the top item from the main stack.
        let item_a = stack_holder.pop()?;

        // Pop the second item from the main stack.
        let item_b = stack_holder.pop()?;

        // Get the bytes
        let a_bytes = item_a.bytes();
        let b_bytes = item_b.bytes();

        // Convert shift amount to usize (safe for byte operations)
        let shift_amount = if b_bytes.is_empty() {
            0
        } else {
            // Use the first byte as shift amount, capped at 8 * a_bytes.len()
            let max_shift = a_bytes.len() * 8;
            let shift = b_bytes[0] as usize;
            if shift > max_shift {
                max_shift
            } else {
                shift
            }
        };

        // Perform byte-based right shift
        let result = if shift_amount == 0 || a_bytes.is_empty() {
            // No shift needed or empty input
            a_bytes.to_vec()
        } else {
            let mut result = Vec::new();
            let byte_shift = shift_amount / 8;
            let bit_shift = shift_amount % 8;

            // Skip bytes that would be shifted out completely
            let start_idx = byte_shift;
            if start_idx >= a_bytes.len() {
                // All bytes would be shifted out
                return Ok(());
            }

            // Handle bit shifting
            if bit_shift == 0 {
                // Just copy the bytes, skipping the ones that would be shifted out
                result.extend_from_slice(&a_bytes[start_idx..]);
            } else {
                // Need to handle bit shifting across byte boundaries
                let mut carry = 0u8;

                // Process bytes from right to left for right shift
                for i in (start_idx..a_bytes.len()).rev() {
                    let byte = a_bytes[i];
                    let shifted = (byte >> bit_shift) | carry;
                    result.insert(0, shifted);
                    carry = byte << (8 - bit_shift);
                }
            }

            result
        };

        // Push the result to the stack
        stack_holder.push(StackItem::new(result))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_RSHIFT_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_RSHIFT` opcode (0x99).
    pub fn bytecode() -> Vec<u8> {
        vec![0x99]
    }
}
