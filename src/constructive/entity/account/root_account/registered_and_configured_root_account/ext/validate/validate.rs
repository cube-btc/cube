use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::validate::validate_error::RegisteredAndConfiguredRootAccountValidateError;

impl RegisteredAndConfiguredRootAccount {
    /// Checks whether the `RegisteredAndConfiguredRootAccount` is indeed a valid registered and configured account.
    ///
    /// NOTE: This validation by nodes is redundant when APE-decoded from the `Registery`.
    pub async fn validate(
        &self,
        registery: &REGISTERY,
    ) -> Result<(), RegisteredAndConfiguredRootAccountValidateError> {
        // 1 Get account info by account key.
        let account_info = {
            // 1.1 Lock the registery.
            let _registery = registery.lock().await;

            // 1.2 Get account info by account key.
            _registery.get_account_info_by_account_key(self.account_key)
        };

        // 2 Check if the account is already registered.
        match account_info {
            // 2.a The account is indeed registered.
            Some((_, bls_key, registery_index, _)) => {
                // 2.a.1 Check if the registery index is the same.
                if registery_index != self.registery_index {
                    return Err(RegisteredAndConfiguredRootAccountValidateError::RegisteryIndexMismatchError);
                }

                // 2.a.2 Check if the BLS key is the same.
                if bls_key != Some(self.bls_key) {
                    return Err(
                        RegisteredAndConfiguredRootAccountValidateError::BLSKeyMismatchError,
                    );
                }

                // 2.a.3 Return Ok.
                Ok(())
            }

            // 2.b The account is not registered.
            None => {
                Err(RegisteredAndConfiguredRootAccountValidateError::AccountIsNotRegisteredError)
            }
        }
    }
}
