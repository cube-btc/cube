use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::unregistered_root_account::ext::validate::validate_error::UnregisteredRootAccountValidateError;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::transmutative::secp::schnorr::Bytes32;

impl UnregisteredRootAccount {
    /// Checks whether the `UnregisteredRootAccount` is indeed a valid unregistered account.
    pub async fn validate(
        &self,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
    ) -> Result<(), UnregisteredRootAccountValidateError> {
        // 1 Get the account key and the BLS key to be configured.
        let (account_key_to_be_registered, bls_key_to_be_configured) = (
            self.account_key_to_be_registered,
            self.bls_key_to_be_configured,
        );

        // 2 Verify that the account key is indeed a valid Schnorr public key.
        if account_key_to_be_registered.to_even_point().is_none() {
            return Err(UnregisteredRootAccountValidateError::InvalidAccountKeyError);
        }

        // 3 Verify that the BLS key is indeed a valid BLS public key.
        {
            // TODO.
        }

        // 4 Check if the account is already registered and BLS key is not conflicting.
        {
            // 4.1 Lock the registery.
            let _registery = registery.lock().await;

            // 4.2 Check if the account is already registered.
            if _registery.is_account_registered(account_key_to_be_registered) {
                return Err(UnregisteredRootAccountValidateError::AccountIsAlreadyRegisteredError);
            }

            // 4.3 Check if the BLS key is not conflicting with an already registered BLS key.
            if _registery
                .bls_key_is_conflicting_with_an_already_registered_bls_key(bls_key_to_be_configured)
            {
                return Err(UnregisteredRootAccountValidateError::BLSKeyIsConflictingWithAnAlreadyRegisteredBLSKeyError);
            }
        }

        // 5 Check if the account has already been burried.
        {
            // 5.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 5.2 Check if the account has already been burried.
            if _graveyard.is_account_burried(account_key_to_be_registered) {
                return Err(UnregisteredRootAccountValidateError::AccountIsAlreadyBurriedError);
            }
        }

        // 6 Verify the authorization signature.
        if !self.verify_authorization_signature() {
            return Err(
                UnregisteredRootAccountValidateError::FailedToVerifyAuthorizationSignatureError,
            );
        }

        // 7 Return Ok.
        Ok(())
    }
}
