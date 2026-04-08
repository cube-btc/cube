use crate::constructive::entry::entries::liftup::ext::validate_lifts::validate_lifts_error::LiftupValidateLiftsError;
use crate::constructive::entry::entries::liftup::liftup::Liftup;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;

impl Liftup {
    /// Checks whether the `Lift`s in the `Liftup` are indeed valid.
    pub async fn validate_lifts(
        &self,
        engine_key: [u8; 32],
        utxo_set: &UTXO_SET,
    ) -> Result<(), LiftupValidateLiftsError> {
        // 1 Validate the structures of the `Lift`s in the `Liftup`.
        {
            // 2.1 Get the self account key.
            let self_account_key = self.account.account_key();

            // 2.2 Validate the structures of the `Lift`s in the `Liftup`.
            for lift in &self.lift_prevtxos {
                if !lift.validate(self_account_key, engine_key) {
                    return Err(LiftupValidateLiftsError::InvalidLiftStructureError);
                }
            }
        }

        // 2 Validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
        {
            // 2.1 Lock the utxo set.
            let _utxo_set = utxo_set.lock().await;

            // 2.2 Validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
            if !_utxo_set.validate_lifts(&self.lift_prevtxos) {
                return Err(LiftupValidateLiftsError::InvalidLiftUTXOError);
            }
        }

        // 3 Return Ok.
        Ok(())
    }
}
