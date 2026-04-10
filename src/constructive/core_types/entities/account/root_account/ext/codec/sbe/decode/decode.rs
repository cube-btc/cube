use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entity::account::root_account::ext::codec::sbe::decode::error::decode_error::RootAccountSBEDecodeError;
use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;

impl RootAccount {
    /// Decodes a `RootAccount` from Structural Byte-scope Encoding (SBE) bytes produced by [`RootAccount::encode_sbe`].
    pub fn decode_sbe(bytes: &[u8]) -> Result<Self, RootAccountSBEDecodeError> {
        // 1 Ensure there is at least one byte for the SBE variant discriminant.
        if bytes.is_empty() {
            return Err(RootAccountSBEDecodeError::RootAccountSBEVariantDiscriminantMissingError);
        }

        // 2 Read the SBE variant discriminant.
        let variant_tag = bytes[0];

        // 3 Slice the payload after the discriminant.
        let payload = &bytes[1..];

        // 4 Match on the SBE variant discriminant.
        match variant_tag {
            // 4.a The `RootAccount` is an `UnregisteredRootAccount`.
            0x00 => {
                // 4.a.1 Ensure the payload holds the Schnorr account key.
                if payload.len() < 32 {
                    return Err(
                        RootAccountSBEDecodeError::UnregisteredRootAccountSBEInsufficientBytesForAccountKey {
                            got_total: bytes.len(),
                        },
                    );
                }

                // 4.a.2 Decode the account key to be registered.
                let account_key_to_be_registered: [u8; 32] = payload[0..32]
                    .try_into()
                    .map_err(|_| {
                        RootAccountSBEDecodeError::UnregisteredRootAccountSBEAccountKeyBytesConversionError
                    })?;

                // 4.a.3 Ensure the payload holds the BLS key.
                if payload.len() < 32 + 48 {
                    return Err(
                        RootAccountSBEDecodeError::UnregisteredRootAccountSBEInsufficientBytesForBlsKey {
                            got_total: bytes.len(),
                        },
                    );
                }

                // 4.a.4 Decode the BLS key to be configured.
                let bls_key_to_be_configured: [u8; 48] = payload[32..32 + 48]
                    .try_into()
                    .map_err(|_| {
                        RootAccountSBEDecodeError::UnregisteredRootAccountSBEBlsKeyBytesConversionError
                    })?;

                // 4.a.5 Decode the flame config to be configured and the authorization signature.
                let tail = &payload[32 + 48..];
                let (flame_config_to_be_configured, authorization_signature) = {
                    // 4.a.5.1 Ensure there is a flame presence prefix byte.
                    let flame_presence = *tail.first().ok_or(
                        RootAccountSBEDecodeError::UnregisteredRootAccountSBEInsufficientBytesForFlamePresencePrefix {
                            got_total: bytes.len(),
                        },
                    )?;

                    // 4.a.5.2 Match on the flame presence prefix.
                    match flame_presence {
                        // 4.a.5.2.a The flame config is not present.
                        0x00 => {
                            // 4.a.5.2.a.1 Ensure the tail holds the authorization signature after the prefix.
                            if tail.len() != 65 {
                                return Err(
                                    RootAccountSBEDecodeError::UnregisteredRootAccountSBEFlameAbsentTrailingSectionLengthError {
                                        got_tail: tail.len(),
                                    },
                                );
                            }

                            // 4.a.5.2.a.2 Decode the authorization signature.
                            let authorization_signature: [u8; 64] = tail[1..65]
                                .try_into()
                                .map_err(|_| {
                                    RootAccountSBEDecodeError::UnregisteredRootAccountSBEFlameAbsentAuthorizationSignatureBytesConversionError
                                })?;

                            // 4.a.5.2.a.3 Return `None` for the flame config and the signature.
                            (None, authorization_signature)
                        }

                        // 4.a.5.2.b The flame config is present.
                        0x01 => {
                            // 4.a.5.2.b.1 Ensure the tail can hold at least one flame byte and the signature.
                            if tail.len() < 1 + 64 {
                                return Err(
                                    RootAccountSBEDecodeError::UnregisteredRootAccountSBEInsufficientBytesWhenFlameConfigPresent {
                                        got_total: bytes.len(),
                                    },
                                );
                            }

                            // 4.a.5.2.b.2 Isolate the flame config bytes and the trailing signature.
                            let sig_start = tail.len() - 64;
                            let flame_config_bytes = &tail[1..sig_start];

                            // 4.a.5.2.b.3 Decode the flame config.
                            let flame_config = FMAccountFlameConfig::from_bytes(flame_config_bytes)
                                .ok_or(
                                    RootAccountSBEDecodeError::UnregisteredRootAccountSBEFailedToDecodeFlameConfigError,
                                )?;

                            // 4.a.5.2.b.4 Decode the authorization signature.
                            let authorization_signature: [u8; 64] = tail[sig_start..]
                                .try_into()
                                .map_err(|_| {
                                    RootAccountSBEDecodeError::UnregisteredRootAccountSBEFlamePresentAuthorizationSignatureBytesConversionError
                                })?;

                            // 4.a.5.2.b.5 Return `Some` for the flame config and the signature.
                            (Some(flame_config), authorization_signature)
                        }

                        // 4.a.5.2.c The flame presence prefix is invalid.
                        b => {
                            return Err(
                                RootAccountSBEDecodeError::UnregisteredRootAccountSBEFlamePresencePrefixInvalidError(
                                    b,
                                ),
                            );
                        }
                    }
                };

                // 4.a.6 Construct the unregistered `RootAccount`.
                let unregistered_root_account = UnregisteredRootAccount::new(
                    account_key_to_be_registered,
                    bls_key_to_be_configured,
                    flame_config_to_be_configured,
                    authorization_signature,
                );

                // 4.a.7 Return the unregistered `RootAccount`.
                Ok(RootAccount::UnregisteredRootAccount(
                    unregistered_root_account,
                ))
            }

            // 4.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            0x01 => {
                // 4.b.1 Ensure the payload holds the Schnorr account key.
                if payload.len() < 32 {
                    return Err(
                        RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForAccountKey {
                            got_total: bytes.len(),
                        },
                    );
                }

                // 4.b.2 Decode the account key.
                let account_key: [u8; 32] = payload[0..32]
                    .try_into()
                    .map_err(|_| {
                        RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEAccountKeyBytesConversionError
                    })?;

                // 4.b.3 Ensure the payload holds the registery index.
                if payload.len() < 32 + 8 {
                    return Err(
                        RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForRegisteryIndex {
                            got_total: bytes.len(),
                        },
                    );
                }

                // 4.b.4 Decode the registery index.
                let registery_index = u64::from_le_bytes(
                    payload[32..40]
                        .try_into()
                        .map_err(|_| {
                            RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBERegisteryIndexBytesConversionError
                        })?,
                );

                // 4.b.5 Ensure the payload holds the BLS key.
                if payload.len() < 32 + 8 + 48 {
                    return Err(
                        RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForBlsKey {
                            got_total: bytes.len(),
                        },
                    );
                }

                // 4.b.6 Decode the BLS key to be configured.
                let bls_key_to_be_configured: [u8; 48] = payload[40..40 + 48]
                    .try_into()
                    .map_err(|_| {
                        RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEBlsKeyBytesConversionError
                    })?;

                // 4.b.7 Decode the flame config to be configured and the authorization signature.
                let tail = &payload[40 + 48..];
                let (flame_config_to_be_configured, authorization_signature) = {
                    // 4.b.7.1 Ensure there is a flame presence prefix byte.
                    let flame_presence = *tail.first().ok_or(
                        RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEInsufficientBytesForFlamePresencePrefix {
                            got_total: bytes.len(),
                        },
                    )?;

                    // 4.b.7.2 Match on the flame presence prefix.
                    match flame_presence {
                        // 4.b.7.2.a The flame config is not present.
                        0x00 => {
                            // 4.b.7.2.a.1 Ensure the tail holds the authorization signature after the prefix.
                            if tail.len() != 65 {
                                return Err(
                                    RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEFlameAbsentTrailingSectionLengthError {
                                        got_tail: tail.len(),
                                    },
                                );
                            }

                            // 4.b.7.2.a.2 Decode the authorization signature.
                            let authorization_signature: [u8; 64] = tail[1..65]
                                .try_into()
                                .map_err(|_| {
                                    RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEFlameAbsentAuthorizationSignatureBytesConversionError
                                })?;

                            // 4.b.7.2.a.3 Return `None` for the flame config and the signature.
                            (None, authorization_signature)
                        }

                        // 4.b.7.2.b The flame config is present.
                        0x01 => {
                            // 4.b.7.2.b.1 Ensure the tail can hold at least one flame byte and the signature.
                            if tail.len() < 1 + 64 {
                                return Err(
                                    RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEInsufficientBytesWhenFlameConfigPresent {
                                        got_total: bytes.len(),
                                    },
                                );
                            }

                            // 4.b.7.2.b.2 Isolate the flame config bytes and the trailing signature.
                            let sig_start = tail.len() - 64;
                            let flame_config_bytes = &tail[1..sig_start];

                            // 4.b.7.2.b.3 Decode the flame config.
                            let flame_config = FMAccountFlameConfig::from_bytes(flame_config_bytes)
                                .ok_or(
                                    RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEFailedToDecodeFlameConfigError,
                                )?;

                            // 4.b.7.2.b.4 Decode the authorization signature.
                            let authorization_signature: [u8; 64] = tail[sig_start..]
                                .try_into()
                                .map_err(|_| {
                                    RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEFlamePresentAuthorizationSignatureBytesConversionError
                                })?;

                            // 4.b.7.2.b.5 Return `Some` for the flame config and the signature.
                            (Some(flame_config), authorization_signature)
                        }

                        // 4.b.7.2.c The flame presence prefix is invalid.
                        b => {
                            return Err(
                                RootAccountSBEDecodeError::RegisteredButUnconfiguredRootAccountSBEFlamePresencePrefixInvalidError(
                                    b,
                                ),
                            );
                        }
                    }
                };

                // 4.b.8 Construct the registered but unconfigured `RootAccount`.
                let registered_but_unconfigured_root_account =
                    RegisteredButUnconfiguredRootAccount::new(
                        account_key,
                        registery_index,
                        bls_key_to_be_configured,
                        flame_config_to_be_configured,
                        authorization_signature,
                    );

                // 4.b.9 Return the registered but unconfigured `RootAccount`.
                Ok(RootAccount::RegisteredButUnconfiguredRootAccount(
                    registered_but_unconfigured_root_account,
                ))
            }

            // 4.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            0x02 => {
                // 4.c.1 Ensure the payload length matches the fixed layout.
                if payload.len() != 32 + 8 + 48 {
                    return Err(
                        RootAccountSBEDecodeError::RegisteredAndConfiguredRootAccountSBEPayloadLengthError {
                            got_payload: payload.len(),
                            expected_payload: 32 + 8 + 48,
                        },
                    );
                }

                // 4.c.2 Decode the account key.
                let account_key: [u8; 32] = payload[0..32]
                    .try_into()
                    .map_err(|_| {
                        RootAccountSBEDecodeError::RegisteredAndConfiguredRootAccountSBEAccountKeyBytesConversionError
                    })?;

                // 4.c.3 Decode the registery index.
                let registery_index = u64::from_le_bytes(
                    payload[32..40]
                        .try_into()
                        .map_err(|_| {
                            RootAccountSBEDecodeError::RegisteredAndConfiguredRootAccountSBERegisteryIndexBytesConversionError
                        })?,
                );

                // 4.c.4 Decode the BLS key.
                let bls_key: [u8; 48] = payload[40..40 + 48]
                    .try_into()
                    .map_err(|_| {
                        RootAccountSBEDecodeError::RegisteredAndConfiguredRootAccountSBEBlsKeyBytesConversionError
                    })?;

                // 4.c.5 Construct the registered and configured `RootAccount`.
                let registered_and_configured_root_account =
                    RegisteredAndConfiguredRootAccount::new(account_key, registery_index, bls_key);

                // 4.c.6 Return the registered and configured `RootAccount`.
                Ok(RootAccount::RegisteredAndConfiguredRootAccount(
                    registered_and_configured_root_account,
                ))
            }

            // 4.d The discriminant does not match any known variant.
            b => {
                Err(RootAccountSBEDecodeError::UnknownRootAccountSBEVariantDiscriminantByteError(b))
            }
        }
    }
}
