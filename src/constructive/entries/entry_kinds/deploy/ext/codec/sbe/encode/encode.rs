use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::constructive::entry::entry_kinds::deploy::ext::codec::sbe::encode::error::encode_error::DeploySBEEncodeError;
use crate::executive::executable::compiler::compiler::ProgramCompiler;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Deploy {
    /// Structural Byte-scope Encoding (SBE) encoding for `Deploy`.
    pub fn encode_sbe(&self) -> Result<Bytes, DeploySBEEncodeError> {
        let root_account_bytes = self.root_account.encode_sbe();
        let root_len_u32 = u32::try_from(root_account_bytes.len()).map_err(|_| {
            DeploySBEEncodeError::DeploySBERootAccountPayloadTooLargeForU32LengthPrefix {
                len: root_account_bytes.len(),
            }
        })?;

        let program_bytes = self
            .program
            .compile()
            .map_err(|_| DeploySBEEncodeError::DeploySBEProgramCompileError)?;
        let program_len_u32 = u32::try_from(program_bytes.len()).map_err(|_| {
            DeploySBEEncodeError::DeploySBEProgramPayloadTooLargeForU32LengthPrefix {
                len: program_bytes.len(),
            }
        })?;

        let mut bytes = Bytes::new();
        bytes.push(0x06);
        bytes.extend_from_slice(&root_len_u32.to_le_bytes());
        bytes.extend_from_slice(&root_account_bytes);
        bytes.extend_from_slice(&program_len_u32.to_le_bytes());
        bytes.extend_from_slice(&program_bytes);
        bytes.extend_from_slice(&self.initial_balance.to_le_bytes());
        bytes.extend_from_slice(&self.target.encode_sbe());
        Ok(bytes)
    }
}
