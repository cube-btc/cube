use crate::constructive::entry::entry_kinds::swapout::ext::pre_validations::validate_overall::validate_overall_error::SwapoutValidateOverallError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;

impl Swapout {
    /// Used by the `Engine` to validate the `Swapout` end-to-end as a pre-validation step before executing it.
    pub async fn validate_overall(
        &self,
        execution_batch_height: u64,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
        coin_manager: &COIN_MANAGER,
        bls_signature: [u8; 96],
    ) -> Result<(), SwapoutValidateOverallError> {
        self.bls_verify(bls_signature)
            .map_err(SwapoutValidateOverallError::ValidateBLSSignatureError)?;

        self.root_account
            .validate_root_account(registery, graveyard)
            .await
            .map_err(|_| SwapoutValidateOverallError::ValidateRootAccountError)?;

        if let Err((targeted_at_batch_height, execution_batch_height_err)) =
            self.target.validate(execution_batch_height)
        {
            return Err(SwapoutValidateOverallError::ValidateTargetError {
                targeted_at_batch_height,
                execution_batch_height: execution_batch_height_err,
            });
        }

        // 4 Ensure only `PinlessSelf::Default` is supported and its location is absent.
        match &self.pinless_self {
            PinlessSelf::Default(pinless_self_default) => {
                if pinless_self_default.location().is_some() {
                    return Err(
                        SwapoutValidateOverallError::SwapoutDefaultPinlessSelfLocationMustBeAbsentError,
                    );
                }
            }
            PinlessSelf::Unknown(_) => {
                return Err(
                    SwapoutValidateOverallError::SwapoutUnknownPinlessSelfNotSupportedYetError,
                );
            }
        }

        let account_balance = {
            let _coin_manager = coin_manager.lock().await;
            _coin_manager
                .get_account_balance(self.root_account.account_key())
                .ok_or(SwapoutValidateOverallError::UnableToReadAccountBalance)?
        };

        if u64::from(self.amount) > account_balance {
            return Err(SwapoutValidateOverallError::InsufficientBalance {
                requested_amount: self.amount,
                account_balance,
            });
        }

        Ok(())
    }
}
