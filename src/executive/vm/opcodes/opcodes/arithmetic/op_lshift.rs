use crate::executive::{
    opcode::ops::OP_LSHIFT_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Shifts a left b bits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_LSHIFT;

impl OP_LSHIFT {
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

        // Perform byte-based left shift
        let result = if shift_amount == 0 {
            // No shift needed
            a_bytes.to_vec()
        } else {
            let mut result = Vec::new();
            let byte_shift = shift_amount / 8;
            let bit_shift = shift_amount % 8;

            // Add leading zeros for byte shift
            for _ in 0..byte_shift {
                result.push(0);
            }

            // Handle bit shifting
            if bit_shift == 0 {
                // Just copy the bytes
                result.extend_from_slice(&a_bytes);
            } else {
                // Need to handle bit shifting across byte boundaries
                let mut carry = 0u8;
                for &byte in a_bytes.iter() {
                    let shifted = (byte << bit_shift) | carry;
                    result.push(shifted);
                    carry = byte >> (8 - bit_shift);
                }
                // Add final carry byte if needed
                if carry != 0 {
                    result.push(carry);
                }
            }

            result
        };

        // Push the result to the stack
        stack_holder.push(StackItem::new(result))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_LSHIFT_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_LSHIFT` opcode (0x98).
    pub fn bytecode() -> Vec<u8> {
        vec![0x98]
    }
}
