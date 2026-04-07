use crate::constructive::entry::entries::liftup::liftup::Liftup;
use crate::constructive::entry::entries::liftup::validate::validate_error::LiftupValidateError;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;

impl Liftup {
    /// Checks whether the `Liftup` is indeed a valid liftup.
    pub async fn validate(
        &self,
        engine_key: [u8; 32],
        utxo_set: &UTXO_SET,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
    ) -> Result<(), LiftupValidateError> {
        // 1 Validate the root account.
        self.account
            .validate(registery, graveyard)
            .await
            .map_err(|e| LiftupValidateError::RootAccountValidationError(e))?;

        // 2 Validate the structures of the `Lift`s in the `Liftup`.
        {
            // 2.1 Get the self account key.
            let self_account_key = self.account.account_key();

            // 2.2 Validate the structures of the `Lift`s in the `Liftup`.
            for lift in &self.lift_prevtxos {
                if !lift.validate(self_account_key, engine_key) {
                    return Err(LiftupValidateError::InvalidLiftStructureError);
                }
            }
        }

        // 3 Validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
        {
            // 3.1 Lock the utxo set.
            let _utxo_set = utxo_set.lock().await;

            // 3.2 Validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
            if !_utxo_set.validate_lifts(&self.lift_prevtxos) {
                return Err(LiftupValidateError::InvalidLiftUTXOError);
            }
        }

        // 4 Return Ok.
        Ok(())
    }
}
