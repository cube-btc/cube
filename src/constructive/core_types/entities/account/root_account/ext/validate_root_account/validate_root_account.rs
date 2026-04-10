use crate::constructive::entity::account::root_account::ext::validate_root_account::validate_root_account_error::RootAccountValidateRootAccountError;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;

impl RootAccount {
    /// Validates that this `RootAccount` matches the `Registery` (and `Graveyard` when unregistered):
    /// cryptographically (keys / authorization) and by registration state (reverse of
    /// [`RootAccount::self_root_account`]).
    pub async fn validate_root_account(
        &self,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
    ) -> Result<(), RootAccountValidateRootAccountError> {
        // 1 Match on the `RootAccount` variant.
        match self {
            // 1.a `UnregisteredRootAccount`
            RootAccount::UnregisteredRootAccount(unregistered_root_account) => {
                // 1.a.1 Validate Schnorr / BLS key material.
                if !unregistered_root_account.validate_schnorr_and_bls_key() {
                    return Err(
                        RootAccountValidateRootAccountError::UnregisteredValidateSchnorrAndBLSKeyError,
                    );
                }

                // 1.a.2 Verify the BLS key authorization signature.
                if !unregistered_root_account.verify_authorization_signature() {
                    return Err(
                        RootAccountValidateRootAccountError::UnregisteredInvalidAuthorizationSignatureError,
                    );
                }

                // 1.a.3 Ensure the account is not burried in the graveyard.
                {
                    // 1.a.3.1 Lock the graveyard.
                    let _graveyard = graveyard.lock().await;

                    // 1.a.3.2 Reject if burried.
                    if _graveyard
                        .is_account_burried(unregistered_root_account.account_key_to_be_registered)
                    {
                        return Err(
                            RootAccountValidateRootAccountError::UnregisteredAccountBurriedInGraveyardError,
                        );
                    }
                }

                // 1.a.4 Ensure the account is not present in the registery.
                {
                    // 1.a.4.1 Lock the registery.
                    let _registery = registery.lock().await;

                    // 1.a.4.2 Reject if registered.
                    if _registery
                        .get_account_info_by_account_key(
                            unregistered_root_account.account_key_to_be_registered,
                        )
                        .is_some()
                    {
                        return Err(
                            RootAccountValidateRootAccountError::UnregisteredAccountRegisteredInRegisteryError,
                        );
                    }
                }

                // 1.a.5 Ok.
                Ok(())
            }

            // 1.b `RegisteredButUnconfiguredRootAccount`
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 1.b.1 Validate the BLS key material.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        RootAccountValidateRootAccountError::RegisteredButUnconfiguredValidateBLSKeyError,
                    );
                }

                // 1.b.2 Verify the BLS key authorization signature.
                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        RootAccountValidateRootAccountError::RegisteredButUnconfiguredInvalidAuthorizationSignatureError,
                    );
                }

                // 1.b.3 Check registery state: registered, index matches, BLS not configured yet.
                {
                    // 1.b.3.1 Lock the registery.
                    let _registery = registery.lock().await;

                    // 1.b.3.2 Get account info by account key.
                    let account_info = _registery
                        .get_account_info_by_account_key(registered_but_unconfigured_root_account.account_key);

                    // 1.b.3.3 Match on account info.
                    match account_info {
                        // 1.b.3.3.a Not registered.
                        None => Err(
                            RootAccountValidateRootAccountError::RegisteredButUnconfiguredAccountNotRegisteredInRegisteryError,
                        ),

                        // 1.b.3.3.b Registered.
                        Some((_account_key, primary_bls_key, registery_index, _rank)) => {
                            // 1.b.3.3.b.1 Registery index must match.
                            if registery_index != registered_but_unconfigured_root_account.registery_index {
                                return Err(
                                    RootAccountValidateRootAccountError::RegisteredButUnconfiguredRegisteryIndexMismatchError,
                                );
                            }

                            // 1.b.3.3.b.2 Primary BLS key must still be unset in the registery.
                            match primary_bls_key {
                                // 1.b.3.3.b.2.a Already configured — inconsistent with this variant.
                                Some(_) => Err(
                                    RootAccountValidateRootAccountError::RegisteredButUnconfiguredBLSKeyAlreadyConfiguredInRegisteryError,
                                ),

                                // 1.b.3.3.b.2.b Not configured — expected.
                                None => Ok(()),
                            }
                        }
                    }
                }
            }

            // 1.c `RegisteredAndConfiguredRootAccount`
            RootAccount::RegisteredAndConfiguredRootAccount(registered_and_configured_root_account) => {
                // 1.c.1 Validate Schnorr / BLS key material.
                if !registered_and_configured_root_account.validate_schnorr_and_bls_key() {
                    return Err(
                        RootAccountValidateRootAccountError::RegisteredAndConfiguredValidateSchnorrAndBLSKeyError,
                    );
                }

                // 1.c.2 Check registery state: registered, index matches, BLS configured and matches.
                {
                    // 1.c.2.1 Lock the registery.
                    let _registery = registery.lock().await;

                    // 1.c.2.2 Get account info by account key.
                    let account_info = _registery
                        .get_account_info_by_account_key(registered_and_configured_root_account.account_key);

                    // 1.c.2.3 Match on account info.
                    match account_info {
                        // 1.c.2.3.a Not registered.
                        None => Err(
                            RootAccountValidateRootAccountError::RegisteredAndConfiguredAccountNotRegisteredInRegisteryError,
                        ),

                        // 1.c.2.3.b Registered.
                        Some((_account_key, primary_bls_key, registery_index, _rank)) => {
                            // 1.c.2.3.b.1 Registery index must match.
                            if registery_index != registered_and_configured_root_account.registery_index {
                                return Err(
                                    RootAccountValidateRootAccountError::RegisteredAndConfiguredRegisteryIndexMismatchError,
                                );
                            }

                            // 1.c.2.3.b.2 Primary BLS key must be set and match this root account.
                            match primary_bls_key {
                                // 1.c.2.3.b.2.a Not configured — inconsistent with this variant.
                                None => Err(
                                    RootAccountValidateRootAccountError::RegisteredAndConfiguredBLSKeyNotConfiguredInRegisteryError,
                                ),

                                // 1.c.2.3.b.2.b Configured — must match.
                                Some(registery_bls_key) => {
                                    if registery_bls_key != registered_and_configured_root_account.bls_key {
                                        return Err(
                                            RootAccountValidateRootAccountError::RegisteredAndConfiguredBLSKeyMismatchWithRegisteryError,
                                        );
                                    }
                                    Ok(())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
