use crate::constructive::entry::entry_types::liftup::ext::validations::validate_overall::validate_overall_error::LiftupValidateOverallError;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;

impl Liftup {
    /// Used by the `Engine` to validate the `Liftup` end-to-end as a pre-validation step before executing it.
    ///
    /// It is redundant for a Node to use this method because all of these valdiations are already done under the hood during APE-decoding.
    pub async fn validate_overall(
        &self,
        engine_key: [u8; 32],
        execution_batch_height: u64,
        utxo_set: &UTXO_SET,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
    ) -> Result<(), LiftupValidateOverallError> {
        // 1 Validate the root account.
        self.root_account
            .validate_root_account(registery, graveyard)
            .await
            .map_err(LiftupValidateOverallError::ValidateRootAccountError)?;

        // 2 Validate the target against the execution batch height.
        if let Err((targeted_at_batch_height, execution_batch_height_err)) =
            self.target.validate(execution_batch_height)
        {
            return Err(LiftupValidateOverallError::ValidateTargetError {
                targeted_at_batch_height,
                execution_batch_height: execution_batch_height_err,
            });
        }

        // 3 Validate the lifts.
        self.validate_lifts(engine_key, utxo_set)
            .await
            .map_err(LiftupValidateOverallError::ValidateLiftsError)?;

        // 4 Ok.
        Ok(())
    }
}
