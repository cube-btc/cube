use crate::constructive::entity::account::root_account::ext::validate::validate_error::RootAccountValidateError;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;

impl RootAccount {
    /// Checks whether the `RootAccount` is indeed a valid root account.
    ///
    /// NOTE: This validation by nodes is redundant when APE-decoded from the `Registery`.
    pub async fn validate(
        &self,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
    ) -> Result<(), RootAccountValidateError> {
        // 1 Match on the `RootAccount` type.
        match self {
            // 1.a The `RootAccount` is an `UnregisteredRootAccount`.
            Self::UnregisteredRootAccount(unregistered_root_account) => {
                unregistered_root_account
                    .validate(registery, graveyard)
                    .await
                    .map_err(|e| {
                        RootAccountValidateError::UnregisteredRootAccountValidateError(e)
                    })?;
            }
            // 1.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            Self::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                registered_but_unconfigured_root_account
                    .validate(registery)
                    .await
                    .map_err(|e| {
                        RootAccountValidateError::RegisteredButUnconfiguredRootAccountValidateError(
                            e,
                        )
                    })?;
            }
            // 1.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            Self::RegisteredAndConfiguredRootAccount(registered_and_configured_root_account) => {
                registered_and_configured_root_account
                    .validate(registery)
                    .await
                    .map_err(|e| {
                        RootAccountValidateError::RegisteredAndConfiguredRootAccountValidateError(e)
                    })?;
            }
        }

        // 2 Return the result.
        Ok(())
    }
}
