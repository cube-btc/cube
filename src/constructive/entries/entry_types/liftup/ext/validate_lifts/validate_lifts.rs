use crate::constructive::entry::entry_types::liftup::ext::validate_lifts::validate_lifts_error::LiftupValidateLiftsError;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;

impl Liftup {
    /// Checks whether the `Lift`s in the `Liftup` are indeed valid.
    pub async fn validate_lifts(
        &self,
        engine_key: [u8; 32],
        utxo_set: &UTXO_SET,
        validate_lifts_with_the_utxo_set: bool,
    ) -> Result<(), LiftupValidateLiftsError> {
        // 1 Validate the structures of the `Lift`s in the `Liftup`.
        {
            // 2.1 Get the self account key.
            let self_account_key = self.root_account.account_key();

            // 2.2 Validate the structures of the `Lift`s in the `Liftup`.
            for lift in &self.lift_prevtxos {
                if !lift.validate_scriptpubkey(self_account_key, engine_key) {
                    return Err(LiftupValidateLiftsError::InvalidLiftScriptpubkeyError(
                        lift.clone(),
                    ));
                }
            }
        }

        // 2 If enabled, validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
        if validate_lifts_with_the_utxo_set {
            {
                // 2.1 Lock the utxo set.
                let _utxo_set = utxo_set.lock().await;

                // 2.2 Validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
                if let Err(invalid_lift) = _utxo_set.validate_lifts(&self.lift_prevtxos) {
                    return Err(
                        LiftupValidateLiftsError::FailedToValidateLiftWithTheUTXOSetError(
                            invalid_lift,
                        ),
                    );
                }
            }
        }

        // 3 Return Ok.
        Ok(())
    }
}
