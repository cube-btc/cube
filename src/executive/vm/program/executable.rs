use super::executable_error::{ExecutableConstructionError, MethodValidationError};
use super::limits::{
    MAX_METHOD_COUNT, MAX_PROGRAM_NAME_LENGTH, MIN_METHOD_COUNT, MIN_PROGRAM_NAME_LENGTH,
};
use super::method::method::ExecutableMethod;
use super::method::method_type::MethodType;
use crate::constructive::valtype::val::atomic_val::atomic_val::AtomicVal;
use crate::executive::executable::compiler::compiler::ExecutableCompiler;
use crate::transmutative::hash::{Hash, HashTag};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;

/// The executable associated with a `Contract`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Executable {
    /// The executable name.
    executable_name: String,
    /// The account key of the deployer.
    deployed_by: [u8; 32],
    /// The methods to execute.
    methods: Vec<ExecutableMethod>,
}

impl Executable {
    /// Constructs a placeholder executable.
    pub fn placeholder_executable() -> Self {
        Self {
            executable_name: String::new(),
            deployed_by: [0x00; 32],
            methods: Vec::new(),
        }
    }

    /// Creates a new `Executable` with the given executable name and list of methods.
    pub fn new(
        executable_name: String,
        deployed_by: [u8; 32],
        methods: Vec<ExecutableMethod>,
    ) -> Result<Self, ExecutableConstructionError> {
        // Check program name length.
        if executable_name.len() > MAX_PROGRAM_NAME_LENGTH
            || executable_name.len() < MIN_PROGRAM_NAME_LENGTH
        {
            return Err(ExecutableConstructionError::ExecutableNameLengthError);
        }

        // Check method count.
        if methods.len() > MAX_METHOD_COUNT || methods.len() < MIN_METHOD_COUNT {
            return Err(ExecutableConstructionError::MethodCountError);
        }

        // Validate the methods.
        Self::validate_methods(&methods)
            .map_err(|e| ExecutableConstructionError::MethodValidationError(e))?;

        // Order the methods.
        let ordered_methods = Self::order_methods(methods);

        // Construct the executable.
        let executable = Self {
            executable_name,
            deployed_by,
            methods: ordered_methods,
        };

        // Return the executable.
        Ok(executable)
    }

    /// Returns the executable name.
    pub fn executable_name(&self) -> &str {
        &self.executable_name
    }

    /// Returns the account key of the deployer.
    pub fn deployed_by(&self) -> [u8; 32] {
        self.deployed_by
    }

    /// Returns the method count.
    pub fn methods_len(&self) -> usize {
        self.methods.len()
    }

    /// Returns the methods.
    pub fn methods(&self) -> &Vec<ExecutableMethod> {
        &self.methods
    }

    /// Returns the method index by given method name.
    pub fn index_by_method_name(&self, method_name: &str) -> Option<usize> {
        self.methods
            .iter()
            .position(|method| method.method_name() == method_name)
    }

    /// Returns the method given the u8 index.
    /// Up to 256 methods are supported per program.
    pub fn method_by_index(&self, index: u8) -> Option<ExecutableMethod> {
        self.methods.get(index as usize).cloned()
    }

    /// Returns the method by given `AtomicVal` index, rather than a u8.
    pub fn method_by_call_method(&self, call_method: AtomicVal) -> Option<ExecutableMethod> {
        let method_index = call_method.value();
        self.method_by_index(method_index)
    }

    /// Orders the methods by prioritizing callable methods first.  
    fn order_methods(methods: Vec<ExecutableMethod>) -> Vec<ExecutableMethod> {
        let mut callable_methods = Vec::<ExecutableMethod>::new();
        let mut non_callable_methods = Vec::<ExecutableMethod>::new();

        for method in methods.iter() {
            if method.method_type() == MethodType::Callable {
                callable_methods.push(method.clone());
            } else {
                non_callable_methods.push(method.clone());
            }
        }

        callable_methods.extend(non_callable_methods);
        callable_methods
    }

    /// Validates the methods.
    fn validate_methods(methods: &Vec<ExecutableMethod>) -> Result<(), MethodValidationError> {
        // Check for duplicate method names.
        {
            let mut method_names = HashSet::<String>::new();
            for method in methods.iter() {
                if !method_names.insert(method.method_name().to_string()) {
                    return Err(MethodValidationError::DuplicateMethodNameError);
                }
            }
        }

        // Check for at least one callable or read-only method type.
        {
            let mut callable_or_read_only_method_type_found = false;
            for method in methods.iter() {
                if method.method_type() == MethodType::Callable
                    || method.method_type() == MethodType::ReadOnly
                {
                    callable_or_read_only_method_type_found = true;
                    break;
                }
            }

            if !callable_or_read_only_method_type_found {
                return Err(MethodValidationError::AllMethodTypesAreInternal);
            }
        }

        Ok(())
    }

    /// Returns the 32-bytes contract ID.
    pub fn contract_id(&self) -> [u8; 32] {
        // Compile the executable.
        let compiled_bytes = match self.compile() {
            Ok(bytes) => bytes,
            Err(_) => return [0xffu8; 32],
        };

        // Get the contract ID hash.
        let contract_id = compiled_bytes.hash(Some(HashTag::ContractID));

        // Return the contract ID.
        contract_id
    }

    /// Returns the executable as a JSON object.
    pub fn json(&self) -> Value {
        // Convert the methods to JSON.
        let methods: Vec<Value> = self.methods.iter().map(|method| method.json()).collect();

        // Construct the executable JSON object.
        let mut obj = Map::new();

        // Add the contract ID to the executable JSON object.
        obj.insert(
            "contract_id".to_string(),
            Value::String(hex::encode(self.contract_id())),
        );

        // Add the deployed by to the program JSON object.
        obj.insert(
            "deployed_by".to_string(),
            Value::String(hex::encode(self.deployed_by)),
        );

        // Add the program name to the program JSON object.
        obj.insert(
            "executable_name".to_string(),
            Value::String(self.executable_name.clone()),
        );

        // Add the methods to the executable JSON object.
        obj.insert("methods".to_string(), Value::Array(methods));

        // Return the program JSON object.
        Value::Object(obj)
    }
}
