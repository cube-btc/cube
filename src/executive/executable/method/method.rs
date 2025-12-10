use super::{
    limits::{
        MAX_METHOD_ARG_COUNT, MAX_METHOD_NAME_LENGTH, MAX_METHOD_OPCODE_COUNT,
        MIN_METHOD_ARG_COUNT, MIN_METHOD_NAME_LENGTH, MIN_METHOD_OPCODE_COUNT,
    },
    method_error::{MethodConstructionError, ScriptValidationError},
    method_type::MethodType,
};
use crate::{
    constructive::calldata::element_type::CalldataElementType,
    executive::{
        opcode::{opcode::Opcode, opcodes::push::op_pushdata::OP_PUSHDATA},
        stack::{
            stack_item::StackItem,
            stack_uint::{SafeConverter, StackItemUintExt},
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A method of an executable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutableMethod {
    /// The method name.
    method_name: String,
    /// The type of method.
    method_type: MethodType,
    /// Call element types.
    arg_types: Vec<CalldataElementType>,
    /// The script to execute.
    script: Vec<Opcode>,
}

impl ExecutableMethod {
    /// Create a new method.
    pub fn new(
        method_name: String,
        method_type: MethodType,
        arg_types: Vec<CalldataElementType>,
        script: Vec<Opcode>,
    ) -> Result<Self, MethodConstructionError> {
        // Check method name byte length.
        if method_name.len() > MAX_METHOD_NAME_LENGTH || method_name.len() < MIN_METHOD_NAME_LENGTH
        {
            return Err(MethodConstructionError::MethodNameLengthError);
        }

        // Check arg count.
        if arg_types.len() > MAX_METHOD_ARG_COUNT || arg_types.len() < MIN_METHOD_ARG_COUNT {
            return Err(MethodConstructionError::ArgCountError);
        }

        // Check opcode count.
        if script.len() > MAX_METHOD_OPCODE_COUNT || script.len() < MIN_METHOD_OPCODE_COUNT {
            return Err(MethodConstructionError::OpcodeCountError);
        }

        // Construct the method.
        let method = Self {
            method_name,
            method_type,
            arg_types,
            script,
        };

        // Validate the script.
        if let Err(e) = method.validate_script() {
            return Err(MethodConstructionError::ScriptValidationError(e));
        }

        // Validate the arg types.
        if !method.validate_arg_types() {
            return Err(MethodConstructionError::ArgValidationError);
        }

        // Return the method.
        Ok(method)
    }

    /// Returns the method name.
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// Returns the method type.
    pub fn method_type(&self) -> MethodType {
        self.method_type
    }

    /// Returns the call element types.
    pub fn arg_types(&self) -> Vec<CalldataElementType> {
        self.arg_types.clone()
    }

    /// Returns the script.
    pub fn script(&self) -> &Vec<Opcode> {
        &self.script
    }

    /// Validates the script.
    pub fn validate_script(&self) -> Result<(), ScriptValidationError> {
        for opcode in self.script.iter() {
            match opcode {
                // Check for non minimal push data.
                Opcode::OP_PUSHDATA(op_pushdata) => {
                    if op_pushdata.0.len() == 1 && !OP_PUSHDATA::check_minimal_push(&op_pushdata.0)
                    {
                        return Err(ScriptValidationError::NonMinimalDataPushError);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Matches the args to the arg types.
    pub fn match_args(&self, args: &Vec<StackItem>) -> bool {
        // Check if the number of args matches the number of arg types.
        if args.len() != self.arg_types.len() {
            return false;
        }

        // Check if the args match the arg types.
        for (i, arg_type) in self.arg_types.iter().enumerate() {
            let arg = match args.get(i) {
                Some(arg) => arg,
                None => return false,
            };

            // Check if the arg matches the arg type by looking at the byte size.
            match arg_type {
                CalldataElementType::U8 => {
                    if arg.len() > 1 {
                        return false;
                    }
                }
                CalldataElementType::U16 => {
                    if arg.len() > 2 {
                        return false;
                    }
                }
                CalldataElementType::U32 => {
                    if arg.len() > 4 {
                        return false;
                    }
                }
                CalldataElementType::U64 => {
                    if arg.len() > 8 {
                        return false;
                    }
                }
                CalldataElementType::Bool => {
                    if arg.len() > 1 {
                        return false;
                    }
                }
                CalldataElementType::Account => {
                    if arg.len() != 32 {
                        return false;
                    }
                }
                CalldataElementType::Contract => {
                    if arg.len() != 32 {
                        return false;
                    }
                }
                CalldataElementType::Bytes(index) => {
                    if arg.len() != *index as u32 + 1 {
                        return false;
                    }
                }
                CalldataElementType::Varbytes => {
                    if arg.len() > 4096 {
                        return false;
                    }
                }
                CalldataElementType::Payable => {
                    if arg.len() > 4 {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Get the payable allocation value.
    pub fn payable_allocation_value(&self, args: &Vec<StackItem>) -> Option<u32> {
        // Get the payable arg value.
        for (index, arg_type) in self.arg_types.iter().enumerate() {
            // Check if the arg type is a payable.
            if *arg_type == CalldataElementType::Payable {
                // Get the payable arg.
                let payable_arg = match args.get(index) {
                    Some(arg) => arg,
                    None => return None,
                };

                // Convert the arg to a `StackUint`.
                let payable_arg_as_stack_uint = match payable_arg.to_stack_uint() {
                    Some(arg) => arg,
                    None => return None,
                };

                // Convert the arg to a u32.
                let payable_arg_as_u32 = match payable_arg_as_stack_uint.to_u32() {
                    Some(arg) => arg,
                    None => return None,
                };

                // Return the arg value.
                return Some(payable_arg_as_u32);
            }
        }

        // No payable arg found.
        None
    }

    /// Validates the args.
    pub fn validate_arg_types(&self) -> bool {
        // More than ONE payable is not allowed.
        self.arg_types
            .iter()
            .filter(|arg| **arg == CalldataElementType::Payable)
            .count()
            <= 1
    }

    /// Returns the method as a JSON object.
    pub fn json(&self) -> Value {
        // Convert the call element types to JSON.
        let arg_types: Vec<Value> = self
            .arg_types
            .iter()
            .map(|element_type| Value::String(element_type.to_string()))
            .collect();

        // Construct the method JSON object.
        let mut obj = Map::new();
        obj.insert(
            "method_name".to_string(),
            Value::String(self.method_name.clone()),
        );

        // Add the method type to the method JSON object.
        obj.insert(
            "method_type".to_string(),
            Value::String(self.method_type.to_string()),
        );

        // Add the call element types to the method JSON object.
        obj.insert("arg_types".to_string(), Value::Array(arg_types));

        // Convert the script to JSON.
        let script: Vec<Value> = self
            .script
            .iter()
            .map(|opcode| Value::String(opcode.to_string()))
            .collect();

        // Add the script to the method JSON object.
        obj.insert("script".to_string(), Value::Array(script));

        // Return the method JSON object.
        Value::Object(obj)
    }
}
