use crate::{
    constructive::calldata::element_type::CalldataElementType,
    executive::{
        executable::method::{
            compiler::compiler_error::{MethodCompileError, MethodDecompileError},
            method::ExecutableMethod,
            method_type::MethodType,
        },
        opcode::{compiler::compiler::OpcodeCompiler, opcode::Opcode},
    },
};

/// A trait for compiling and decompiling a executable method.
pub trait MethodCompiler {
    /// Compiles the method into a bytecode.
    fn compile(&self) -> Result<Vec<u8>, MethodCompileError>;
    /// Decompiles a method from a bytecode stream.
    fn decompile<I>(bytecode_stream: &mut I) -> Result<ExecutableMethod, MethodDecompileError>
    where
        I: Iterator<Item = u8>;
}

impl MethodCompiler for ExecutableMethod {
    fn compile(&self) -> Result<Vec<u8>, MethodCompileError> {
        // Compile the script.
        let mut method_bytes = Vec::<u8>::new();

        // Encode method name byte length as u8.
        method_bytes.push(self.method_name().len() as u8);

        // Encode method name.
        method_bytes.extend(self.method_name().as_bytes());

        // Encode method type.
        method_bytes.push(self.method_type().bytecode());

        // Encode the number of arg types as u8.
        method_bytes.push(self.arg_types().len() as u8);

        // Encode individual arg types.
        for arg_type in self.arg_types().iter() {
            method_bytes.extend(arg_type.bytecode());
        }

        // Get the number of opcodes as u16.
        let opcodes_count = self.script().len() as u16;

        // Encode the number of opcodes.
        method_bytes.extend(opcodes_count.to_le_bytes());

        // Encode individual opcodes.
        for opcode in self.script().iter() {
            method_bytes.extend(
                opcode
                    .compile()
                    .map_err(|e| MethodCompileError::OpcodeCompileError(e))?,
            );
        }

        // Return the bytecode.
        Ok(method_bytes)
    }

    fn decompile<I>(mut bytecode_stream: &mut I) -> Result<ExecutableMethod, MethodDecompileError>
    where
        I: Iterator<Item = u8>,
    {
        // Collect method name length
        let method_name_length = bytecode_stream
            .next()
            .ok_or(MethodDecompileError::NameLengthByteCollectError)?;

        // Collect method name
        let method_name_bytes: Vec<u8> = bytecode_stream
            .by_ref()
            .take(method_name_length as usize)
            .collect();

        // Check if the method name bytes length is equal to the method name length.
        if method_name_bytes.len() != method_name_length as usize {
            return Err(MethodDecompileError::NameBytesCollectError);
        }

        // Convert method name bytes to string.
        let method_name = String::from_utf8_lossy(&method_name_bytes).to_string();

        // Collect method type bytecode.
        let method_type_bytecode = bytecode_stream
            .by_ref()
            .next()
            .ok_or(MethodDecompileError::MethodTypeByteCollectError)?;

        // Get method type from the bytecode.
        let method_type = MethodType::from_bytecode(method_type_bytecode)
            .ok_or(MethodDecompileError::InvalidMethodType)?;

        // Collect the number of call element types.
        let number_of_call_element_types = bytecode_stream
            .by_ref()
            .next()
            .ok_or(MethodDecompileError::NumberOfCallElementTypesByteCollectError)?;

        // Collect call element types.
        let mut calldata_element_types = Vec::<CalldataElementType>::new();
        for _ in 0..number_of_call_element_types {
            let call_element_type = CalldataElementType::from_bytecode(&mut bytecode_stream)
                .ok_or(MethodDecompileError::InvalidCallElementType)?;

            calldata_element_types.push(call_element_type);
        }

        // Collect two bytes for opcodes count.
        let opcodes_count = u16::from_le_bytes([
            bytecode_stream.by_ref().next().unwrap(),
            bytecode_stream.by_ref().next().unwrap(),
        ]);

        // Collect opcodes.
        let mut opcodes = Vec::<Opcode>::new();
        for _ in 0..opcodes_count {
            let opcode = Opcode::decompile(&mut bytecode_stream)
                .map_err(|e| MethodDecompileError::OpcodeDecompileError(e))?;

            opcodes.push(opcode);
        }

        // Construct the method.
        let method = ExecutableMethod::new(method_name, method_type, calldata_element_types, opcodes)
            .map_err(|e| MethodDecompileError::MethodConstructError(e))?;

        // Return the method.
        Ok(method)
    }
}
