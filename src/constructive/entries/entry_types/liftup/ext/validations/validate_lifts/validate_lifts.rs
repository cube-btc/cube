use crate::constructive::entry::entry_types::liftup::ext::validations::validate_lifts::validate_lifts_error::LiftupValidateLiftsError;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::constructive::txo::lift::lift::Lift;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;

impl Liftup {
    /// Used by the `Engine` to check if the `Lift`s in a `Liftup` are indeed valid.
    pub async fn validate_lifts(
        &self,
        engine_key: [u8; 32],
        utxo_set: &UTXO_SET,
    ) -> Result<(), LiftupValidateLiftsError> {
        // 1 Validate the structures of the `Lift`s in the `Liftup`.
        {
            // 1 Validate the structures of the `Lift`s in the `Liftup`.
            for lift in &self.lift_prevtxos {
                match lift {
                    Lift::LiftV1(v1) => {
                        if !v1.validate_scriptpubkey() {
                            return Err(LiftupValidateLiftsError::InvalidLiftScriptpubkeyError(
                                lift.clone(),
                            ));
                        }
                    }
                    Lift::LiftV2(v2) => {
                        if !v2.validate_scriptpubkey() {
                            return Err(LiftupValidateLiftsError::InvalidLiftScriptpubkeyError(
                                lift.clone(),
                            ));
                        }
                    }
                    Lift::Unknown { .. } => {}
                }
            }
        }

        // 2 Validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
        {
            // 2.1 Lock the utxo set.
            let _utxo_set = utxo_set.lock().await;

            // 2.2 Validate the `Lift`s in the `Liftup` are indeed valid UTXOs.
            if let Err(invalid_lift) = _utxo_set.validate_lifts(&self.lift_prevtxos) {
                return Err(
                    LiftupValidateLiftsError::FailedToValidateLiftWithTheUTXOSetError(invalid_lift),
                );
            }
        }

        // 3 Return Ok.
        Ok(())
    }
}
