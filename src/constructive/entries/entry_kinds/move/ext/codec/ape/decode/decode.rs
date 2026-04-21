use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::r#move::ext::codec::ape::decode::error::decode_error::MoveAPEDecodeError;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery::registery::REGISTERY;

impl Move {
    /// Decodes a `Move` as an Airly Payload Encoding (APE) bit vector.
    pub async fn decode_ape(
        execution_batch_height: u64,
        bit_stream: &mut bit_vec::Iter<'_>,
        decode_account_rank_as_longval: bool,
        registery: &REGISTERY,
    ) -> Result<Move, MoveAPEDecodeError> {
        // 1 Decode the sender `RootAccount`.
        let from = RootAccount::decode_ape(bit_stream, decode_account_rank_as_longval, registery)
            .await
            .map_err(MoveAPEDecodeError::RootAccountAPEDecodeError)?;

        // 2 Decode the receiver `Account`.
        let to = Account::decode_ape(bit_stream, registery, decode_account_rank_as_longval)
            .await
            .map_err(MoveAPEDecodeError::AccountAPEDecodeError)?;

        // 3 Decode the amount.
        let amount = ShortVal::decode_ape(bit_stream)
            .map_err(MoveAPEDecodeError::AmountAPEDecodeError)?
            .value();

        // 4 Decode the `Target`.
        let target = Target::decode_ape(bit_stream, execution_batch_height)
            .map_err(MoveAPEDecodeError::TargetAPEDecodeError)?;

        // 5 Construct and return the decoded `Move`.
        Ok(Move::new(from, to, amount, target))
    }
}
