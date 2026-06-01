use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::constructive::entry::entry_kinds::deploy::ext::codec::ape::encode::error::encode_error::DeployAPEEncodeError;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::executive::executable::compiler::compiler::ProgramCompiler;
use crate::inscriptive::registry::registry::REGISTRY;
use bit_vec::BitVec;

impl Deploy {
    /// Airly Payload Encoding (APE) encoding for `Deploy`.
    pub async fn encode_ape(
        &self,
        execution_batch_height: u64,
        registry: &REGISTRY,
        encode_account_rank_as_longval: bool,
    ) -> Result<BitVec, DeployAPEEncodeError> {
        let mut bits = BitVec::new();

        let root_bits = self
            .root_account
            .encode_ape(registry, encode_account_rank_as_longval)
            .await
            .map_err(DeployAPEEncodeError::RootAccountAPEEncodeError)?;
        bits.extend(root_bits);

        let program_bytes = self
            .program
            .compile()
            .map_err(|_| DeployAPEEncodeError::ProgramCompileError)?;
        let program_len =
            u32::try_from(program_bytes.len()).map_err(|_| DeployAPEEncodeError::ProgramLenTooLarge(program_bytes.len()))?;
        bits.extend(ShortVal::new(program_len).encode_ape());
        bits.extend(BitVec::from_bytes(&program_bytes));

        bits.extend(BitVec::from_bytes(&self.initial_balance.to_le_bytes()));

        let target_bits = self
            .target
            .encode_ape(execution_batch_height)
            .map_err(DeployAPEEncodeError::TargetAPEEncodeError)?;
        bits.extend(target_bits);

        Ok(bits)
    }
}
