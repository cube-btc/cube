use crate::executive::stack::{
    stack_error::{StackError, StackUintError},
    stack_holder::StackHolder,
    stack_uint::StackItemUintExt,
};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};

/// The `OP_GOV_ACCOUNT` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_GOV_ACCOUNT;

impl OP_GOV_ACCOUNT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the gov account kind
        let gov_account_kind = stack_holder.pop()?;

        if gov_account_kind.len() != 1 {
            return Err(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ));
        }

        // Into stack uint
        let gov_account_kind_as_stack_uint =
            gov_account_kind
                .to_stack_uint()
                .ok_or(StackError::StackUintError(
                    StackUintError::StackUintConversionError,
                ))?;

        // Match on the gov account kind
        match gov_account_kind_as_stack_uint.as_usize() {
            // Freeze pleb init
            0 => {}
            // Freeze pleb undo
            1 => {}
            // Expel pleb init
            2 => {}
            // Expel pleb undo
            3 => {}
            // Turn account resident
            4 => {}
            // Turn account citizen
            5 => {}
            // Update account privileges
            6 => {}
            // Permit account dev
            7 => {}
            // Permit account lp
            9 => {}
            _ => {
                return Err(StackError::StackUintError(
                    StackUintError::StackUintConversionError,
                ));
            }
        }

        stack_holder.increment_ops(OpcodeOpsParams::as_u32(
            stack_holder.opcode_ops().op_gov_account,
        ))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_GOV_ACCOUNT` opcode (0xd4).
    pub fn bytecode() -> Vec<u8> {
        vec![0xd4]
    }
}
