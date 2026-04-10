use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entries::entry_types::liftup::ape::decode::error::decode_error::LiftupAPEDecodeError;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::constructive::txo::lift::lift::Lift;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use bitcoin::OutPoint;

impl Liftup {
    /// Decodes a `Liftup` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function decodes a `Liftup` as an Airly Payload Encoding (APE) bit vector.
    /// The `Liftup` can be a `Liftup` with a `Account`, `Lifts`.
    pub async fn decode_ape(
        engine_key: [u8; 32],
        execution_batch_height: u64,
        bit_stream: &mut bit_vec::Iter<'_>,
        tx_inputs_iter: &mut impl Iterator<Item = OutPoint>,
        decode_account_rank_as_longval: bool,
        utxo_set: &UTXO_SET,
        registery: &REGISTERY,
    ) -> Result<Liftup, LiftupAPEDecodeError> {
        // 1 Decode the `RootAccount` from the bitstream.
        let root_account: RootAccount =
            RootAccount::decode_ape(bit_stream, decode_account_rank_as_longval, registery)
                .await
                .map_err(|e| LiftupAPEDecodeError::RootAccountAPEDecodeError(e))?;

        // 2 Decode the `Target` from the bitstream.
        let target: Target = Target::decode_ape(bit_stream, execution_batch_height)
            .map_err(|e| LiftupAPEDecodeError::TargetAPEDecodeError(e))?;

        // 3 Decode the number of lifts.
        let number_of_lifts: u32 = ShortVal::decode_ape(bit_stream)
            .map_err(|e| LiftupAPEDecodeError::NumberOfLiftsAPEDecodeError(e))?
            .value();

        // 4 Collect the outpoints for each lift.
        let mut collected_outpoints = Vec::with_capacity(number_of_lifts as usize);
        for _ in 0..number_of_lifts {
            let outpoint = tx_inputs_iter
                .next()
                .ok_or(LiftupAPEDecodeError::MissingLiftOutpointError)?;
            collected_outpoints.push(outpoint);
        }

        // 5 Collect one-bit lift kind tags (0 => v1, 1 => v2).
        let mut lift_kind_bits = Vec::with_capacity(number_of_lifts as usize);
        for _ in 0..number_of_lifts {
            let lift_kind = bit_stream
                .next()
                .ok_or(LiftupAPEDecodeError::MissingLiftKindBitError)?;
            lift_kind_bits.push(lift_kind);
        }

        // 6 Resolve the outpoints in the UTXO set and construct the lifts.
        let self_account_key = root_account.account_key();
        let mut lift_prevtxos = Vec::<Lift>::with_capacity(number_of_lifts as usize);
        {
            let _utxo_set = utxo_set.lock().await;

            for (outpoint, is_v2_lift) in collected_outpoints.into_iter().zip(lift_kind_bits) {
                let txout = _utxo_set.txout_by_outpoint(&outpoint).ok_or(
                    LiftupAPEDecodeError::UnableToLocateLiftOutpointInUTXOSetError(outpoint),
                )?;

                let lift = if is_v2_lift {
                    Lift::new_liftv2(self_account_key, engine_key, outpoint, txout)
                } else {
                    Lift::new_liftv1(self_account_key, engine_key, outpoint, txout)
                };

                lift_prevtxos.push(lift);
            }
        }

        // 7 Construct and return the decoded `Liftup`.
        let liftup = Liftup::new(root_account, target, lift_prevtxos);

        // 8 Return the decoded `Liftup`.
        Ok(liftup)
    }
}
