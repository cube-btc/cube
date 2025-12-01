use super::compiler_error::{ExecutableCompileError, ExecutableDecompileError};
use crate::executive::executable::executable::Executable;
use crate::executive::executable::method::{
    compiler::compiler::MethodCompiler, method::ExecutableMethod,
};

/// A trait for compiling and decompiling a executable.
pub trait ExecutableCompiler {
    /// Compiles the executable into a bytecode.
    fn compile(&self) -> Result<Vec<u8>, ExecutableCompileError>;
    /// Decompiles a executable from a bytecode stream.
    fn decompile<I>(bytecode_stream: &mut I) -> Result<Executable, ExecutableDecompileError>
    where
        I: Iterator<Item = u8>;
}

impl ExecutableCompiler for Executable {
    fn compile(&self) -> Result<Vec<u8>, ExecutableCompileError> {
        // Compile the script.
        let mut executable_bytes = Vec::<u8>::new();

        // Encode program name byte length as u8.
        executable_bytes.push(self.executable_name().len() as u8);

        // Encode program name.
        executable_bytes.extend(self.executable_name().as_bytes());

        // Encode deployed by.
        executable_bytes.extend(self.deployed_by());

        // Encode method count as u8.
        executable_bytes.push(self.methods_len() as u8);

        // Encode methods.
        for method in self.methods().iter() {
            executable_bytes.extend(
                method
                    .compile()
                    .map_err(|e| ExecutableCompileError::MethodCompileError(e))?,
            );
        }

        // Return the bytecode.
        Ok(executable_bytes)
    }

    fn decompile<I>(bytecode_stream: &mut I) -> Result<Executable, ExecutableDecompileError>
    where
        I: Iterator<Item = u8>,
    {
        // Decode executable name byte length.
        let executable_name_byte_length = bytecode_stream
            .next()
            .ok_or(ExecutableDecompileError::NameLengthByteCollectError)?;

        // Collect executable name bytes.
        let executable_name_bytes: Vec<u8> = bytecode_stream
            .by_ref()
            .take(executable_name_byte_length as usize)
            .collect();

        // Check if the executable name bytes length is equal to the executable name byte length.
        if executable_name_bytes.len() != executable_name_byte_length as usize {
            return Err(ExecutableDecompileError::ExecutableNameBytesCollectError);
        }

        // Collect 32 byte for the account key of the deployer.
        let deployed_by: [u8; 32] = bytecode_stream
            .by_ref()
            .take(32)
            .collect::<Vec<u8>>()
            .try_into()
            .map_err(|_| ExecutableDecompileError::DeployedByBytesCollectError)?;

        // Convert executable name bytes to string.
        let executable_name = String::from_utf8_lossy(&executable_name_bytes).to_string();

        // Collect method count byte.
        let method_count = bytecode_stream
            .next()
            .ok_or(ExecutableDecompileError::MethodCountByteCollectError)?;

        // Collect methods.
        let mut methods = Vec::<ExecutableMethod>::new();
        for _ in 0..method_count {
            let method = ExecutableMethod::decompile(bytecode_stream)
                .map_err(|e| ExecutableDecompileError::MethodDecompileError(e))?;
            methods.push(method);
        }

        // Construct the executable.
        let executable = Executable::new(executable_name, deployed_by, methods)
            .map_err(|e| ExecutableDecompileError::ExecutableConstructError(e))?;

        // Return the executable.
        Ok(executable)
    }
}
