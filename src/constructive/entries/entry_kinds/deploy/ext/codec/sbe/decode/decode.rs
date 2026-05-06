use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::constructive::entry::entry_kinds::deploy::ext::codec::sbe::decode::error::decode_error::DeploySBEDecodeError;
use crate::executive::executable::compiler::compiler::ProgramCompiler;
use crate::executive::executable::executable::Program;

impl Deploy {
    /// Decodes a `Deploy` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Deploy, DeploySBEDecodeError> {
        if bytes.is_empty() {
            return Err(DeploySBEDecodeError::DeploySBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: 0,
            });
        }
        if bytes[0] != 0x06 {
            return Err(DeploySBEDecodeError::InvalidEntryKindByteError {
                expected: 0x06,
                got: bytes[0],
            });
        }

        if bytes.len() < 5 {
            return Err(DeploySBEDecodeError::DeploySBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: bytes.len(),
            });
        }
        let root_len = u32::from_le_bytes(
            bytes[1..5]
                .try_into()
                .map_err(|_| DeploySBEDecodeError::DeploySBERootAccountLengthPrefixBytesConversionError)?,
        ) as usize;
        let after_root_len_prefix = &bytes[5..];
        if after_root_len_prefix.len() < root_len {
            return Err(DeploySBEDecodeError::DeploySBERootAccountLengthPrefixExceedsPayload {
                root_len,
                got_after_prefix: after_root_len_prefix.len(),
            });
        }
        let (root_slice, mut tail) = after_root_len_prefix.split_at(root_len);
        let root_account = RootAccount::decode_sbe(root_slice)
            .map_err(DeploySBEDecodeError::DeploySBERootAccountDecodeError)?;

        if tail.len() < 4 {
            return Err(DeploySBEDecodeError::DeploySBEInsufficientBytesForProgramLengthPrefix {
                got_total: bytes.len(),
            });
        }
        let program_len = u32::from_le_bytes(
            tail[0..4]
                .try_into()
                .map_err(|_| DeploySBEDecodeError::DeploySBEProgramLengthPrefixBytesConversionError)?,
        ) as usize;
        tail = &tail[4..];
        if tail.len() < program_len {
            return Err(DeploySBEDecodeError::DeploySBEProgramLengthPrefixExceedsPayload {
                program_len,
                got_after_prefix: tail.len(),
            });
        }
        let (program_slice, tail_after_program) = tail.split_at(program_len);
        let program = Program::decompile(&mut program_slice.iter().cloned())
            .map_err(|_| DeploySBEDecodeError::DeploySBEProgramDecompileError)?;
        tail = tail_after_program;

        if tail.len() < 4 {
            return Err(DeploySBEDecodeError::DeploySBEInsufficientBytesForInitialBalance {
                got_total: bytes.len(),
            });
        }
        let initial_balance = u32::from_le_bytes(
            tail[0..4]
                .try_into()
                .map_err(|_| DeploySBEDecodeError::DeploySBEInitialBalanceBytesConversionError)?,
        );
        tail = &tail[4..];

        if tail.len() < 8 {
            return Err(DeploySBEDecodeError::DeploySBEInsufficientBytesForTarget {
                got_total: bytes.len(),
            });
        }
        let target = Target::decode_sbe(&tail[0..8]).map_err(DeploySBEDecodeError::DeploySBETargetDecodeError)?;
        tail = &tail[8..];

        if !tail.is_empty() {
            return Err(DeploySBEDecodeError::DeploySBETrailingBytesAfterDeploy {
                trailing: tail.len(),
            });
        }

        Ok(Deploy::new(root_account, program, initial_balance, target))
    }
}
