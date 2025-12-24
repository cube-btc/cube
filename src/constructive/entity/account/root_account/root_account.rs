use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use crate::transmutative::key::KeyHolder;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum RootAccount {
    // A fresh, unregistered (thus unranked), and unconfigured account.
    UnregisteredRootAccount(UnregisteredRootAccount),

    // A registered but unconfigured root account.
    RegisteredButUnconfiguredRootAccount(RegisteredButUnconfiguredRootAccount),

    // A registered and configured root account.
    RegisteredAndConfiguredRootAccount(RegisteredAndConfiguredRootAccount),
}

impl RootAccount {
    /// Returns the `RootAccount` for the given `KeyHolder`.
    pub async fn self_root_account(
        keyholder: &KeyHolder,
        registery_manager: &REGISTERY_MANAGER,
    ) -> Option<Self> {
        // 1 Get the self account key.
        let self_account_key: [u8; 32] = keyholder.secp_public_key_bytes();

        // 2 Get the self BLS key.
        let self_bls_key: [u8; 48] = keyholder.bls_public_key_bytes();

        // 3 Lock the registery manager.
        let _registery_manager = registery_manager.lock().await;

        // 4 Check if the self account is registered.
        let is_registered = _registery_manager.is_account_registered(self_account_key);

        // 5 Match on whether the account is registered or not.
        match is_registered {
            // 5.a The account is registered.
            true => {
                // 5.a.1 Get the BLS key by the account key.
                let bls_key = _registery_manager.get_bls_key_by_account_key(self_account_key);

                // 5.a.2 Match on whether the BLS key is configured or not.
                match bls_key {
                    // 5.a.2.a The BLS key is configured.
                    Some(bls_key) => {
                        // 5.a.2.a.1 Construct the `RegisteredAndConfiguredRootAccount`.
                        let registered_and_configured_root_account =
                            RegisteredAndConfiguredRootAccount::new(self_account_key, bls_key);

                        // 5.a.2.a.2 Construct the `RootAccount`.
                        let root_account = Self::RegisteredAndConfiguredRootAccount(
                            registered_and_configured_root_account,
                        );

                        // 5.a.2.a.3 Return the `RootAccount`.
                        Some(root_account)
                    }

                    // 5.a.2.b The BLS key is not configured.
                    None => {
                        // TODO:
                        let authentication_signature = [0u8; 64];

                        // 5.a.2.b.1 Construct the `RegisteredButUnconfiguredRootAccount`.
                        let registered_but_unconfigured_root_account =
                            RegisteredButUnconfiguredRootAccount::new(
                                self_account_key,
                                self_bls_key,
                                None,
                                authentication_signature,
                            );

                        // 5.a.2.b.2 Construct the `RootAccount`.
                        let root_account = Self::RegisteredButUnconfiguredRootAccount(
                            registered_but_unconfigured_root_account,
                        );

                        // 5.a.2.b.3 Return the `RootAccount`.
                        Some(root_account)
                    }
                }
            }

            // 5.b The account is not registered.
            false => {
                // TODO:
                let authentication_signature = [0u8; 64];

                // 5.b.1 Construct the `UnregisteredRootAccount`.
                let unregistered_root_account = UnregisteredRootAccount::new(
                    self_account_key,
                    self_bls_key,
                    None,
                    authentication_signature,
                );

                // 5.b.2 Construct the `RootAccount`.
                let root_account = Self::UnregisteredRootAccount(unregistered_root_account);

                // 5.b.3 Return the `RootAccount`.
                Some(root_account)
            }
        }
    }

    /// Returns whether the `RootAccount` is registered.
    pub fn is_registered(&self) -> bool {
        match self {
            Self::RegisteredButUnconfiguredRootAccount(_) => true,
            Self::RegisteredAndConfiguredRootAccount(_) => true,
            _ => false,
        }
    }

    /// Returns the account key of the `RootAccount`.
    pub fn account_key(&self) -> [u8; 32] {
        match self {
            Self::UnregisteredRootAccount(unregistered_root_account) => {
                unregistered_root_account.account_key_to_be_registered
            }
            Self::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => registered_but_unconfigured_root_account.account_key,
            Self::RegisteredAndConfiguredRootAccount(registered_and_configured_root_account) => {
                registered_and_configured_root_account.account_key
            }
        }
    }
}

impl PartialEq for RootAccount {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UnregisteredRootAccount(left), Self::UnregisteredRootAccount(right)) => {
                left == right
            }
            (
                Self::RegisteredButUnconfiguredRootAccount(left),
                Self::RegisteredButUnconfiguredRootAccount(right),
            ) => left == right,
            (
                Self::RegisteredAndConfiguredRootAccount(left),
                Self::RegisteredAndConfiguredRootAccount(right),
            ) => left == right,
            _ => false,
        }
    }
}

impl Eq for RootAccount {}
