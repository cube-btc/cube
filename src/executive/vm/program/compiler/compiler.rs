use super::compiler_error::{ProgramCompileError, ProgramDecompileError};
use crate::executive::executable::executable::Program;
use crate::executive::executable::method::{
    compiler::compiler::MethodCompiler, program_method::ProgramMethod,
};

/// A trait for compiling and decompiling a program.
pub trait ProgramCompiler {
    /// Compiles the program into a bytecode.
    fn compile(&self) -> Result<Vec<u8>, ProgramCompileError>;
    /// Decompiles a program from a bytecode stream.
    fn decompile<I>(bytecode_stream: &mut I) -> Result<Program, ProgramDecompileError>
    where
        I: Iterator<Item = u8>;
}

impl ProgramCompiler for Program {
    fn compile(&self) -> Result<Vec<u8>, ProgramCompileError> {
        // Compile the script.
        let mut executable_bytes = Vec::<u8>::new();

        // Encode program name byte length as u8.
        executable_bytes.push(self.program_name().len() as u8);

        // Encode program name.
        executable_bytes.extend(self.program_name().as_bytes());

        // Encode metadata.
        match self.metadata() {
            Some(metadata) => {
                executable_bytes.push(0x01);
                executable_bytes.extend((metadata.len() as u16).to_le_bytes());
                executable_bytes.extend(metadata);
            }
            None => executable_bytes.push(0x00),
        }

        // Encode method count as u8.
        executable_bytes.push(self.methods_len() as u8);

        // Encode methods.
        for method in self.methods().iter() {
            executable_bytes.extend(
                method
                    .compile()
                    .map_err(|e| ProgramCompileError::MethodCompileError(e))?,
            );
        }

        // Return the bytecode.
        Ok(executable_bytes)
    }

    fn decompile<I>(bytecode_stream: &mut I) -> Result<Program, ProgramDecompileError>
    where
        I: Iterator<Item = u8>,
    {
        // Decode executable name byte length.
        let executable_name_byte_length = bytecode_stream
            .next()
            .ok_or(ProgramDecompileError::NameLengthByteCollectError)?;

        // Collect executable name bytes.
        let executable_name_bytes: Vec<u8> = bytecode_stream
            .by_ref()
            .take(executable_name_byte_length as usize)
            .collect();

        // Check if the executable name bytes length is equal to the executable name byte length.
        if executable_name_bytes.len() != executable_name_byte_length as usize {
            return Err(ProgramDecompileError::ProgramNameBytesCollectError);
        }

        // Collect metadata presence byte.
        let metadata = match bytecode_stream
            .next()
            .ok_or(ProgramDecompileError::MetadataFlagByteCollectError)?
        {
            0x00 => None,
            0x01 => {
                let metadata_len_bytes: [u8; 2] = bytecode_stream
                    .by_ref()
                    .take(2)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .map_err(|_| ProgramDecompileError::MetadataLengthBytesCollectError)?;
                let metadata_len = u16::from_le_bytes(metadata_len_bytes) as usize;
                let metadata_bytes: Vec<u8> =
                    bytecode_stream.by_ref().take(metadata_len).collect::<Vec<u8>>();
                if metadata_bytes.len() != metadata_len {
                    return Err(ProgramDecompileError::MetadataBytesCollectError);
                }
                Some(metadata_bytes)
            }
            _ => return Err(ProgramDecompileError::MetadataFlagByteCollectError),
        };

        // Convert executable name bytes to string.
        let program_name = String::from_utf8_lossy(&executable_name_bytes).to_string();

        // Collect method count byte.
        let method_count = bytecode_stream
            .next()
            .ok_or(ProgramDecompileError::MethodCountByteCollectError)?;

        // Collect methods.
        let mut methods = Vec::<ProgramMethod>::new();
        for _ in 0..method_count {
            let method = ProgramMethod::decompile(bytecode_stream)
                .map_err(|e| ProgramDecompileError::MethodDecompileError(e))?;
            methods.push(method);
        }

        // Construct the executable.
        let executable = Program::new(program_name, metadata, methods)
            .map_err(|e| ProgramDecompileError::ProgramConstructError(e))?;

        // Return the executable.
        Ok(executable)
    }
}
