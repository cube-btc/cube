use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::account::ext::validate_account::validate_account_error::AccountValidateAccountError;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registry::registry::REGISTRY;

impl Account {
    /// Validates an `Account`.
    pub async fn validate_account(
        &self,
        registry: &REGISTRY,
        graveyard: &GRAVEYARD,
    ) -> Result<(), AccountValidateAccountError> {
        // 1 Match on the `Account` variant.
        match self {
            // 1.a `UnregisteredAccount`
            Account::UnregisteredAccount(unregistered_account) => {
                // 1.a.1 Validate Schnorr key material.
                if !unregistered_account.validate_schnorr_key() {
                    return Err(AccountValidateAccountError::UnregisteredValidateSchnorrKeyError);
                }

                // 1.a.2 Ensure the account is not buried in the graveyard.
                {
                    // 1.a.2.1 Lock the graveyard.
                    let _graveyard = graveyard.lock().await;

                    // 1.a.2.2 Reject if buried.
                    if _graveyard
                        .is_account_buried(unregistered_account.account_key_to_be_registered)
                    {
                        return Err(
                            AccountValidateAccountError::UnregisteredAccountBuriedInGraveyardError,
                        );
                    }
                }

                // 1.a.3 Ensure the account is not present in the registry.
                {
                    // 1.a.3.1 Lock the registry.
                    let _registry = registry.lock().await;

                    // 1.a.3.2 Reject if registered.
                    if _registry
                        .get_account_info_by_account_key(
                            unregistered_account.account_key_to_be_registered,
                        )
                        .is_some()
                    {
                        return Err(
                            AccountValidateAccountError::UnregisteredAccountRegisteredInRegistryError,
                        );
                    }
                }

                // 1.a.4 Ok.
                Ok(())
            }

            // 1.b `RegisteredAccount`
            Account::RegisteredAccount(registered_account) => {
                // 1.b.1 Check registry state: registered and index matches.
                {
                    // 1.b.1.1 Lock the registry.
                    let _registry = registry.lock().await;

                    // 1.b.1.2 Get account info by account key.
                    let account_info =
                        _registry.get_account_info_by_account_key(registered_account.account_key);

                    // 1.b.1.3 Match on account info.
                    match account_info {
                        // 1.b.1.3.a Not registered.
                        None => Err(
                            AccountValidateAccountError::RegisteredAccountNotRegisteredInRegistryError,
                        ),

                        // 1.b.1.3.b Registered.
                        Some((_account_key, _primary_bls_key, registry_index, _rank)) => {
                            // 1.b.1.3.b.1 Registry index must match.
                            if registry_index != registered_account.registry_index {
                                return Err(
                                    AccountValidateAccountError::RegisteredRegistryIndexMismatchError,
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
