use crate::constructive::entry::entry_kinds::r#move::ext::codec::ape::encode::error::encode_error::MoveAPEEncodeError;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery::registery::REGISTERY;
use bit_vec::BitVec;

impl Move {
    /// Airly Payload Encoding (APE) encoding for `Move`.
    pub async fn encode_ape(
        &self,
        execution_batch_height: u64,
        registery: &REGISTERY,
        encode_account_rank_as_longval: bool,
    ) -> Result<BitVec, MoveAPEEncodeError> {
        // 1 Initialize the bit vector.
        let mut bits = BitVec::new();

        // 2 Encode the sender `RootAccount`.
        {
            let from_bits = self
                .from
                .encode_ape(registery, encode_account_rank_as_longval)
                .await
                .map_err(MoveAPEEncodeError::RootAccountAPEEncodeError)?;
            bits.extend(from_bits);
        }

        // 3 Encode the receiver `Account`.
        {
            let to_bits = self
                .to
                .encode_ape(registery, encode_account_rank_as_longval)
                .await
                .map_err(MoveAPEEncodeError::AccountAPEEncodeError)?;
            bits.extend(to_bits);
        }

        // 4 Encode the amount.
        {
            let amount_as_shortval = ShortVal::new(self.amount);
            bits.extend(amount_as_shortval.encode_ape());
        }

        // 5 Encode the `Target`.
        {
            let target_bits = self
                .target
                .encode_ape(execution_batch_height)
                .map_err(MoveAPEEncodeError::TargetAPEEncodeError)?;
            bits.extend(target_bits);
        }

        // 6 Return the bit vector.
        Ok(bits)
    }
}
