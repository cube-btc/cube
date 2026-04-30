use crate::constructive::entry::entry_kinds::swapout::ext::codec::ape::encode::error::encode_error::SwapoutAPEEncodeError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::inscriptive::registery::registery::REGISTERY;

impl Swapout {
    /// Airly Payload Encoding (APE) encoding for `Swapout`.
    pub async fn encode_ape(
        &self,
        execution_batch_height: u64,
        registery: &REGISTERY,
        encode_account_rank_as_longval: bool,
    ) -> Result<bit_vec::BitVec, SwapoutAPEEncodeError> {
        let mut bits = bit_vec::BitVec::new();

        let root_account_bits = self
            .root_account
            .encode_ape(registery, encode_account_rank_as_longval)
            .await
            .map_err(SwapoutAPEEncodeError::RootAccountAPEEncodeError)?;
        bits.extend(root_account_bits);

        let target_bits = self
            .target
            .encode_ape(execution_batch_height)
            .map_err(SwapoutAPEEncodeError::TargetAPEEncodeError)?;
        bits.extend(target_bits);

        match self.pinless_self {
            PinlessSelf::Default(_) => bits.push(false),
            PinlessSelf::Unknown(_) => bits.push(true),
        }

        Ok(bits)
    }
}
