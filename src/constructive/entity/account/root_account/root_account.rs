use crate::{constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount, inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig};
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use crate::transmutative::key::KeyHolder;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use serde::{Deserialize, Serialize};
use crate::transmutative::hash::Hash;
use crate::transmutative::hash::HashTag;
use crate::transmutative::secp::schnorr;
use crate::transmutative::secp::schnorr::SchnorrSigningMode;

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
    ) -> RootAccount {
        // 1 Get the self account key.
        let self_account_key: [u8; 32] = keyholder.secp_public_key_bytes();

        // 2 Get the self BLS key.
        let self_bls_key: [u8; 48] = keyholder.bls_public_key_bytes();

        // 3 Get the self flame config.
        // NOTE: Set to `None` for now.
        let self_flame_config: Option<FMAccountFlameConfig> = None;

        // 4 Retrieve the account info if its registered.
        let account_info = {
            // 4.1 Lock the registery manager.
            let _registery_manager = registery_manager.lock().await;

            // 4.2 Get account info by account key.
            _registery_manager.get_account_info_by_account_key(self_account_key)
        };

        // 5 Match on whether the account is registered or not.
        match account_info {
            // 5.a The account is registered.
            Some((_, bls_key, registery_index, _)) => {
                // 5.a.1 Match on whether the BLS key is configured or not.
                match bls_key {
                    // 5.a.1.a The BLS key is configured.
                    Some(bls_key) => {
                        // 5.a.1.a.1 Construct the `RegisteredAndConfiguredRootAccount`.
                        let registered_and_configured_root_account =
                            RegisteredAndConfiguredRootAccount::new(
                                self_account_key,
                                registery_index,
                                bls_key,
                            );

                        // 5.a.1.a.2 Construct the `RootAccount`.
                        let root_account = Self::RegisteredAndConfiguredRootAccount(
                            registered_and_configured_root_account,
                        );

                        // 5.a.1.a.3 Return the `RootAccount`.
                        root_account
                    }

                    // 5.a.1.b The BLS key is not configured.
                    None => {
                        // 5.a.1.b.1
                        let authorization_signature = produce_bls_key_authorization_signature(
                            keyholder.secp_secret_key_bytes(),
                            self_account_key,
                            self_bls_key,
                            &None,
                        )
                        .expect("This should never happen: Failed to produce the BLS key authorization signature.");

                        // 5.a.1.b.2 Construct the `RegisteredButUnconfiguredRootAccount`.
                        let registered_but_unconfigured_root_account =
                            RegisteredButUnconfiguredRootAccount::new(
                                self_account_key,
                                registery_index,
                                self_bls_key,
                                self_flame_config,
                                authorization_signature,
                            );

                        // 5.a.1.b.3 Construct the `RootAccount`.
                        let root_account = Self::RegisteredButUnconfiguredRootAccount(
                            registered_but_unconfigured_root_account,
                        );

                        // 5.a.1.b.4 Return the `RootAccount`.
                        root_account
                    }
                }
            }

            // 5.b The account is not registered.
            None => {
                // 5.b.1 Produce the BLS key authorization signature.
                let authorization_signature = produce_bls_key_authorization_signature(
                    keyholder.secp_secret_key_bytes(),
                    self_account_key,
                    self_bls_key,
                    &self_flame_config,
                )
                .expect("This should never happen: Failed to produce the BLS key authorization signature.");

                // 5.b.2 Construct the `UnregisteredRootAccount`.
                let unregistered_root_account = UnregisteredRootAccount::new(
                    self_account_key,
                    self_bls_key,
                    self_flame_config,
                    authorization_signature,
                );

                // 5.b.3 Construct the `RootAccount`.
                let root_account = Self::UnregisteredRootAccount(unregistered_root_account);

                // 5.b.4 Return the `RootAccount`.
                root_account
            }
        }
    }

    /// Returns whether the `RootAccount` is registered.
    pub fn is_registered(&self) -> bool {
        // 1 Match on the `RootAccount` type.
        match self {
            // 1.a The `RootAccount` is an `UnregisteredRootAccount`.
            Self::RegisteredButUnconfiguredRootAccount(_) => true,

            // 1.b The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            Self::RegisteredAndConfiguredRootAccount(_) => true,

            // 1.c The `RootAccount` is not registered.
            _ => false,
        }
    }

    /// Returns the `RootAccount`'s Schnorr account key.
    pub fn account_key(&self) -> [u8; 32] {
        // 1 Match on the `RootAccount` type.
        match self {
            // 1.a The `RootAccount` is an `UnregisteredRootAccount`.
            Self::UnregisteredRootAccount(unregistered_root_account) => {
                unregistered_root_account.account_key_to_be_registered
            }
            // 1.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            Self::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => registered_but_unconfigured_root_account.account_key,

            // 1.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            Self::RegisteredAndConfiguredRootAccount(registered_and_configured_root_account) => {
                registered_and_configured_root_account.account_key
            }
        }
    }

    /// Returns the `RootAccount`'s BLS key.
    pub fn bls_key(&self) -> [u8; 48] {
        // 1 Match on the `RootAccount` type.
        match self {
            // 1.a The `RootAccount` is an `UnregisteredRootAccount`.
            Self::UnregisteredRootAccount(unregistered_root_account) => {
                unregistered_root_account.bls_key_to_be_configured
            }
            // 1.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            Self::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => registered_but_unconfigured_root_account.bls_key_to_be_configured,

            // 1.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            Self::RegisteredAndConfiguredRootAccount(registered_and_configured_root_account) => {
                registered_and_configured_root_account.bls_key
            }
        }
    }

    /// Checks whether the `RootAccount` is indeed a valid `RootAccount`.
    pub async fn validate(&self, registery_manager: &REGISTERY_MANAGER) -> bool {
        // 1 Match on the `RootAccount` type.
        match self {
            // 1.a The `RootAccount` is an `UnregisteredRootAccount`.
            Self::UnregisteredRootAccount(unregistered_root_account) => {
                unregistered_root_account.validate(registery_manager).await
            }

            // 1.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            Self::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                registered_but_unconfigured_root_account
                    .validate(registery_manager)
                    .await
            }

            // 1.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            Self::RegisteredAndConfiguredRootAccount(registered_and_configured_root_account) => {
                registered_and_configured_root_account
                    .validate(registery_manager)
                    .await
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

/// Constructs the BLS key authorization message.
pub fn bls_key_authorization_message(
    account_key: [u8; 32],
    bls_key: [u8; 48],
    flame_config: &Option<FMAccountFlameConfig>,
) -> [u8; 32] {
    // 1 Construct the preimage.
    let mut preimage = Vec::<u8>::with_capacity(32 + 48 + 32);

    // 2 Extend the preimage with the account key.
    preimage.extend(account_key);

    // 3 Extend the preimage with the bls key.
    preimage.extend(bls_key);

    // 4 Get the flame config hash.
    let flame_config_hash: [u8; 32] = match &flame_config {
        // 4.a The flame config is present.
        Some(flame_config) => flame_config.hash(),

        // 4.b The flame config is not present.
        None => [0x00u8; 32],
    };

    // 5 Extend the preimage with the flame config hash.
    preimage.extend(flame_config_hash);

    // 6 Hash the preimage to get the message.
    let message = preimage.hash(Some(HashTag::BLSKeyAuthorizationMessage));

    // 7 Return the message.
    message
}

/// Verifies the BLS key authorization signature.
pub fn verify_bls_key_authorization_signature(
    account_key: [u8; 32],
    bls_key: [u8; 48],
    flame_config: &Option<FMAccountFlameConfig>,
    signature: [u8; 64],
) -> bool {
    // 1 Get the BLS key authorization message.
    let message = bls_key_authorization_message(account_key, bls_key, flame_config);

    // 2 Verify the BLS key authorization signature.
    schnorr::verify_xonly(account_key, message, signature, SchnorrSigningMode::Cube)
}

/// Produces the BLS key authorization signature.
pub fn produce_bls_key_authorization_signature(
    secret_key: [u8; 32],
    account_key: [u8; 32],
    bls_key: [u8; 48],
    flame_config: &Option<FMAccountFlameConfig>,
) -> Option<[u8; 64]> {
    // 1 Get the BLS key authorization message.
    let message = bls_key_authorization_message(account_key, bls_key, flame_config);

    // 2 Sign the BLS key authorization message.
    schnorr::sign(secret_key, message, SchnorrSigningMode::Cube)
}
