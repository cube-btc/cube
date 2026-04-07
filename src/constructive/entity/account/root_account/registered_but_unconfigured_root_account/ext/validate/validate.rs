use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::validate::validate_error::RegisteredButUnconfiguredRootAccountValidateError;

impl RegisteredButUnconfiguredRootAccount {
    /// Checks whether the `RegisteredButUnconfiguredRootAccount` is indeed a valid registered but unconfigured account.
    pub async fn validate(
        &self,
        registery: &REGISTERY,
    ) -> Result<(), RegisteredButUnconfiguredRootAccountValidateError> {
        // 1 Get the BLS key to be configured.
        let bls_key_to_be_configured = self.bls_key_to_be_configured;

        // 2 Verify that the BLS key is indeed a valid BLS public key.
        {
            // TODO.
        }

        // 3 Get account info by account key and check if the BLS key is not conflicting.
        let account_info = {
            // 3.1 Lock the registery.
            let _registery = registery.lock().await;

            // 3.2 Get account info by account key.
            let account_info = _registery.get_account_info_by_account_key(self.account_key);

            // 3.3 Check if the BLS key is not conflicting.
            if _registery
                .bls_key_is_conflicting_with_an_already_registered_bls_key(bls_key_to_be_configured)
            {
                return Err(
                    RegisteredButUnconfiguredRootAccountValidateError::BLSKeyIsConflictingError,
                );
            }

            // 3.4 Return the account info.
            account_info
        };

        // 4 Check if the account is already registered.
        match account_info {
            // 4.a The account is indeed registered.
            Some((_, bls_key, registery_index, _)) => {
                // 4.a.1 Check if the registery index is the same.
                if registery_index != self.registery_index {
                    return Err(RegisteredButUnconfiguredRootAccountValidateError::RegisteryIndexMismatchError);
                }

                // 4.a.2 Check if the BLS key is indeed not configured.
                if bls_key.is_some() {
                    return Err(RegisteredButUnconfiguredRootAccountValidateError::BLSKeyIsAlreadyConfiguredError);
                }
            }

            // 4.b The account is not registered.
            None => {
                return Err(
                    RegisteredButUnconfiguredRootAccountValidateError::AccountIsNotRegisteredError,
                )
            }
        }

        // 5 Verify the authorization signature.
        if !self.verify_authorization_signature() {
            return Err(RegisteredButUnconfiguredRootAccountValidateError::FailedToVerifyAuthorizationSignatureError);
        }

        // 6 Return Ok.
        Ok(())
    }
}
