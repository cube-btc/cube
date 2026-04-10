use crate::constructive::entity::account::root_account::root_account::RootAccount;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl RootAccount {
    /// Structural Byte-scope Encoding (SBE) encoding for `RootAccount`.
    ///
    /// This function encodes a `RootAccount` in non-compact, byte-scope format akin to bincode::serialize.
    pub fn encode_sbe(&self) -> Bytes {
        // 1 Initialize the byte vector.
        let mut bytes = Bytes::new();

        // 2 Match on the `RootAccount` variant.
        match self {
            // 2.a The `RootAccount` is a `UnregisteredRootAccount`.
            RootAccount::UnregisteredRootAccount(unregistered_root_account) => {
                // 2.a.1 Push 0x00 to indicate that the `RootAccount` is a `UnregisteredRootAccount`.
                bytes.push(0x00);

                // 2.a.2 Encode the account key to be registered.
                bytes.extend(unregistered_root_account.account_key_to_be_registered);

                // 2.a.3 Encode the BLS key to be configured.
                bytes.extend(unregistered_root_account.bls_key_to_be_configured);

                // 2.a.4 Encode the flame config to be configured.
                match &unregistered_root_account.flame_config_to_be_configured {
                    // 2.a.4.a The flame config is present.
                    Some(flame_config) => {
                        // 2.a.4.a.1 Push a true bit to indicate that the flame config is present.
                        bytes.push(0x01);

                        // 2.a.4.a.2 Serialize the flame config to bytes.
                        bytes.extend(flame_config.to_bytes());
                    }
                    // 2.a.4.b The flame config is not present.
                    None => {
                        // 2.a.4.b.1 Push a false bit to indicate that the flame config is not present.
                        bytes.push(0x00);
                    }
                }

                // 2.a.5 Encode the authorization signature.
                bytes.extend(unregistered_root_account.authorization_signature);
            }
            // 2.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 2.b.1 Push 0x01 to indicate that the `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
                bytes.push(0x01);

                // 2.b.2 Encode the account key.
                bytes.extend(registered_but_unconfigured_root_account.account_key);

                // 2.b.3 Encode the registery index.
                bytes.extend(
                    registered_but_unconfigured_root_account
                        .registery_index
                        .to_le_bytes(),
                );

                // 2.b.4 Encode the BLS key to be configured.
                bytes.extend(registered_but_unconfigured_root_account.bls_key_to_be_configured);

                // 2.b.5 Encode the flame config to be configured.
                match &registered_but_unconfigured_root_account.flame_config_to_be_configured {
                    // 2.b.5.a The flame config is present.
                    Some(flame_config) => {
                        // 2.b.5.a.1 Push a true bit to indicate that the flame config is present.
                        bytes.push(0x01);

                        // 2.b.5.a.2 Serialize the flame config to bytes.
                        bytes.extend(flame_config.to_bytes());
                    }
                    // 2.b.5.b The flame config is not present.
                    None => {
                        // 2.b.5.b.1 Push a false bit to indicate that the flame config is not present.
                        bytes.push(0x00);
                    }
                }

                // 2.b.6 Encode the authorization signature.
                bytes.extend(registered_but_unconfigured_root_account.authorization_signature);
            }
            // 2.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 2.c.1 Push 0x02 to indicate that the `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
                bytes.push(0x02);

                // 2.c.2 Encode the account key.
                bytes.extend(registered_and_configured_root_account.account_key);

                // 2.c.3 Encode the registery index.
                bytes.extend(
                    registered_and_configured_root_account.registery_index.to_le_bytes(),
                );

                // 2.c.4 Encode the BLS key.
                bytes.extend(registered_and_configured_root_account.bls_key);
            }
        }

        // 3 Return the bytes.
        bytes
    }
}
