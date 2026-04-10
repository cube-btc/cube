use crate::constructive::entry::entry_types::liftup::ext::codec::ape::encode::error::encode_error::LiftupAPEEncodeError;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery::registery::REGISTERY;
use bit_vec::BitVec;

impl Liftup {
    /// Airly Payload Encoding (APE) encoding for `Liftup`.
    ///
    /// This function encodes a `Liftup` as an Airly Payload Encoding (APE) bit vector.
    pub async fn encode_ape(
        &self,
        execution_batch_height: u64,
        registery: &REGISTERY,
        encode_account_rank_as_longval: bool,
    ) -> Result<BitVec, LiftupAPEEncodeError> {
        // 1 Initialize the bit vector.
        let mut bits = BitVec::new();

        // 2 Encode the `RootAccount`.
        {
            // 2.1 Encode the `RootAccount`.
            let root_account_bits = self
                .root_account
                .encode_ape(registery, encode_account_rank_as_longval)
                .await
                .map_err(|e| LiftupAPEEncodeError::RootAccountAPEEncodeError(e))?;

            // 2.2 Extend the bit vector with the `RootAccount` bit vector.
            bits.extend(root_account_bits);
        }

        // 3 Encode the `Target`.
        {
            // 3.1 Encode the `Target`.
            let target_bits = self.target.encode_ape(execution_batch_height)
                .map_err(|e| LiftupAPEEncodeError::TargetAPEEncodeError(e))?;

            // 3.2 Extend the bit vector with the `Target` bit vector.
            bits.extend(target_bits);
        }

        // 4 Encode the number of lifts.
        {
            // 4.1 Get the number of lifts.
            let number_of_lifts = self.lift_prevtxos.len();

            // 4.2 Convert the number of lifts into a `ShortVal`.
            let number_of_lifts_as_shortval = ShortVal::new(number_of_lifts as u32);

            // 4.3 Extend the bit vector with the number of lifts.
            bits.extend(number_of_lifts_as_shortval.encode_ape());
        }

        // 5 Encode one-bit lift kind tags (0 => v1, 1 => v2).
        {
            for lift in &self.lift_prevtxos {
                bits.push(lift.lift_version() == 2);
            }
        }

        // 6 Return the bit vector.
        Ok(bits)
    }
}
