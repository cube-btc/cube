use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::constructive::entry::entry_kinds::deploy::ext::codec::ape::decode::error::decode_error::DeployAPEDecodeError;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::executive::executable::compiler::compiler::ProgramCompiler;
use crate::executive::executable::executable::Program;
use crate::inscriptive::registry::registry::REGISTRY;
use bit_vec::BitVec;

impl Deploy {
    /// Decodes a `Deploy` as an Airly Payload Encoding (APE) bit vector.
    pub async fn decode_ape(
        execution_batch_height: u64,
        bit_stream: &mut bit_vec::Iter<'_>,
        decode_account_rank_as_longval: bool,
        registry: &REGISTRY,
    ) -> Result<Deploy, DeployAPEDecodeError> {
        let root_account =
            RootAccount::decode_ape(bit_stream, decode_account_rank_as_longval, registry)
                .await
                .map_err(DeployAPEDecodeError::RootAccountAPEDecodeError)?;

        let program_len = ShortVal::decode_ape(bit_stream)
            .map_err(DeployAPEDecodeError::ProgramLenDecodeError)?
            .value() as usize;
        let program_bits: BitVec = bit_stream.by_ref().take(program_len * 8).collect();
        if program_bits.len() != program_len * 8 {
            return Err(DeployAPEDecodeError::ProgramBitsCollectError);
        }
        let program_bytes = program_bits.to_bytes();
        let program = Program::decompile(&mut program_bytes.into_iter())
            .map_err(|_| DeployAPEDecodeError::ProgramDecompileError)?;

        let initial_balance_bits: BitVec = bit_stream.by_ref().take(32).collect();
        if initial_balance_bits.len() != 32 {
            return Err(DeployAPEDecodeError::InitialBalanceBitsCollectError);
        }
        let initial_balance_bytes = initial_balance_bits.to_bytes();
        let initial_balance = u32::from_le_bytes(
            initial_balance_bytes
                .as_slice()
                .try_into()
                .map_err(|_| DeployAPEDecodeError::InitialBalanceBytesConversionError)?,
        );

        let target = Target::decode_ape(bit_stream, execution_batch_height)
            .map_err(DeployAPEDecodeError::TargetAPEDecodeError)?;

        Ok(Deploy::new(root_account, program, initial_balance, target))
    }
}
