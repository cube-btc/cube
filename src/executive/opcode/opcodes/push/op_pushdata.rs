use super::op_10::OP_10;
use super::op_11::OP_11;
use super::op_12::OP_12;
use super::op_13::OP_13;
use super::op_14::OP_14;
use super::op_15::OP_15;
use super::op_16::OP_16;
use super::op_2::OP_2;
use super::op_3::OP_3;
use super::op_4::OP_4;
use super::op_5::OP_5;
use super::op_6::OP_6;
use super::op_7::OP_7;
use super::op_8::OP_8;
use super::op_9::OP_9;
use super::op_false::OP_FALSE;
use super::op_true::OP_TRUE;
use crate::executive::stack::{
    limits::MAX_STACK_ITEM_SIZE, stack_error::StackError, stack_holder::StackHolder,
    stack_item::StackItem,
};
use serde::{Deserialize, Serialize};

/// Pushes data to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_PUSHDATA(pub Vec<u8>);

impl OP_PUSHDATA {
    /// Executes the push data opcode.
    pub fn execute(&self, stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // The data to be pushed is the inner data in the OP_PUSHDATA struct.
        let item_to_push = StackItem::new(self.0.to_owned());

        // Get the data length.
        let data_len = item_to_push.len();

        // Check if the data length is valid.
        if data_len > MAX_STACK_ITEM_SIZE {
            return Err(StackError::StackItemTooLarge);
        }

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        // Increment the ops counter.
        stack_holder.increment_ops(calculate_ops(data_len))?;

        Ok(())
    }

    /// Check if the push data is a minimal push.
    pub fn check_minimal_push(data: &[u8]) -> bool {
        match data.len() {
            // Data should have been encoded as OP_FALSE (OP_0).
            0 => false,
            // Data might or might not be a minimal push.
            1 => {
                // Check if the data is a minimal push.
                match data.get(0) {
                    // Should have been encoded as OP_0..OP_16.
                    Some(value) if value >= &0 && value <= &16 => false,
                    // Validation passes otherwise.
                    _ => true,
                }
            }
            // Minimal push values are always single byte values.
            // Check for multi-byte values are not needed.
            _ => true,
        }
    }

    /// Returns the compiled bytes for the push data.
    pub fn compiled_bytes(&self) -> Option<Vec<u8>> {
        {
            // Match data length.
            match self.0.len() {
                0 => Some(OP_FALSE::bytecode()),
                1 => {
                    // Get the value.
                    let value: u8 = self.0[0];

                    // Check value for minimal encoding.
                    match value {
                        0 => Some(OP_FALSE::bytecode()),
                        1 => Some(OP_TRUE::bytecode()),
                        2 => Some(OP_2::bytecode()),
                        3 => Some(OP_3::bytecode()),
                        4 => Some(OP_4::bytecode()),
                        5 => Some(OP_5::bytecode()),
                        6 => Some(OP_6::bytecode()),
                        7 => Some(OP_7::bytecode()),
                        8 => Some(OP_8::bytecode()),
                        9 => Some(OP_9::bytecode()),
                        10 => Some(OP_10::bytecode()),
                        11 => Some(OP_11::bytecode()),
                        12 => Some(OP_12::bytecode()),
                        13 => Some(OP_13::bytecode()),
                        14 => Some(OP_14::bytecode()),
                        15 => Some(OP_15::bytecode()),
                        16 => Some(OP_16::bytecode()),
                        _ => {
                            // Initialize the encoded vector.
                            let mut encoded = Vec::<u8>::with_capacity(self.0.len() + 1);

                            // Push the data length.
                            encoded.push(self.0.len() as u8);

                            // Push the data.
                            encoded.extend(self.0.clone());

                            // Return the encoded vector.
                            Some(encoded)
                        }
                    }
                }
                2..=75 => {
                    // Initialize the encoded vector.
                    let mut encoded = Vec::<u8>::with_capacity(self.0.len() + 1);

                    // Push the data length.
                    encoded.push(self.0.len() as u8);

                    // Push the data.
                    encoded.extend(self.0.clone());

                    // Return the encoded vector.
                    Some(encoded)
                }
                76..=255 => {
                    // Initialize the encoded vector.
                    let mut encoded = Vec::<u8>::with_capacity(self.0.len() + 2);

                    // Push 0x4c to the encoded vector.
                    encoded.push(0x4c);

                    // Push the data length.
                    encoded.push(self.0.len() as u8);

                    // Push the data.
                    encoded.extend(self.0.clone());

                    // Return the encoded vector.
                    Some(encoded)
                }
                256..=65535 => {
                    // Initialize the encoded vector.
                    let mut encoded = Vec::<u8>::with_capacity(self.0.len() + 3);

                    // Push 0x4d to the encoded vector.
                    encoded.push(0x4d);

                    // Push the data length as a little endian u16.
                    let data_length = self.0.len() as u16;
                    encoded.push((data_length & 0xff) as u8);
                    encoded.push((data_length >> 8) as u8);

                    // Push the data.
                    encoded.extend(self.0.clone());

                    // Return the encoded vector.
                    Some(encoded)
                }
                _ => None,
            }
        }
    }
}

const PUSHDATA_OPS_BASE: u32 = 1;
const PUSHDATA_OPS_MULTIPLIER: u32 = 1;

// Calculate the number of ops for a push data opcode.
fn calculate_ops(data_len: u32) -> u32 {
    PUSHDATA_OPS_BASE + (PUSHDATA_OPS_MULTIPLIER * data_len)
}
