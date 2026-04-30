use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::swapout::ext::codec::ape::decode::error::decode_error::SwapoutAPEDecodeError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::inscriptive::registery::registery::REGISTERY;
use bitcoin::{OutPoint, TxOut};

impl Swapout {
    /// Decodes a `Swapout` as an Airly Payload Encoding (APE) bit vector.
    pub async fn decode_ape(
        execution_batch_height: u64,
        bit_stream: &mut bit_vec::Iter<'_>,
        tx_outputs_iter: &mut impl Iterator<Item = (OutPoint, TxOut)>,
        decode_account_rank_as_longval: bool,
        registery: &REGISTERY,
    ) -> Result<Self, SwapoutAPEDecodeError> {
        let root_account = RootAccount::decode_ape(bit_stream, decode_account_rank_as_longval, registery)
            .await
            .map_err(|_| SwapoutAPEDecodeError::RootAccountAPEDecodeError)?;
        let target = Target::decode_ape(bit_stream, execution_batch_height)
            .map_err(|_| SwapoutAPEDecodeError::TargetAPEDecodeError)?;
        let default_or_unknown_bit = bit_stream
            .next()
            .ok_or(SwapoutAPEDecodeError::PinlessSelfKindBitCollectError)?;

        let (outpoint, txout) = tx_outputs_iter
            .next()
            .ok_or(SwapoutAPEDecodeError::SwapoutTxOutputCollectError)?;
        let amount = u32::try_from(txout.value.to_sat())
            .map_err(|_| SwapoutAPEDecodeError::SwapoutTxOutputCollectError)?;

        let pinless_self = if default_or_unknown_bit {
            PinlessSelf::new_unknown(txout.script_pubkey.as_bytes().to_vec(), Some((outpoint, txout)))
        } else {
            PinlessSelf::new_default(root_account.account_key(), Some((outpoint, txout)))
        };

        Ok(Swapout::new(root_account, amount, target, pinless_self))
    }
}
