use crate::constructive::entity::account::root_account::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use crate::transmutative::hash::{Hash, HashTag};
use crate::transmutative::secp::schnorr::{self, Bytes32, SchnorrSigningMode};
use bit_vec::BitVec;

impl RootAccount {
    /// Decodes a `RootAccount` from an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function decodes a `RootAccount` from an Airly Payload Encoding (APE) bit vector.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `registery_manager` - The `Registery Manager`.
    /// * `decode_rank_as_longval` - Whether to decode the rank value as a `LongVal` or a `ShortVal`.
    pub async fn decode_ape<'a>(
        bit_stream: &mut bit_vec::Iter<'a>,
        registery_manager: &REGISTERY_MANAGER,
        decode_rank_as_longval: bool,
    ) -> Result<RootAccount, RootAccountAPEDecodeError> {
        // 1 Decode the rank value from the APE bitstream.
        let rank: u64 = match decode_rank_as_longval {
            // 1.a The rank is decoded as a `LongVal`.
            true => {
                // 1.a.1 Decode the rank value as a `LongVal`.
                let rank = LongVal::decode_ape(bit_stream)
                    .map_err(|e| RootAccountAPEDecodeError::FailedToDecodeRankValueAsLongVal(e))?
                    .value();

                // 1.a.2 Return the rank value as a `u64`.
                rank
            }

            // 1.b The rank is decoded as a `ShortVal`.
            false => {
                // 1.b.1 Decode the rank value as a `ShortVal`.
                let rank = ShortVal::decode_ape(bit_stream)
                    .map_err(|e| RootAccountAPEDecodeError::FailedToDecodeRankValueAsShortVal(e))?
                    .value();

                // 1.b.2 Return the rank value as a `u64`.
                rank as u64
            }
        };

        // 2 Match on the rank value to determine if the `RootAccount` is registered or not.
        match rank {
            // 2.a The `RootAccount` is unregistered.
            0 => {
                // 2.a.1 Decode the account key from the APE bitstream.
                let account_key_bytes: [u8; 32] = {
                    // 2.a.1.1 Collect exactly 256 bits for the `RootAccount`'s schnorr account key.
                    let account_key_bits: BitVec = bit_stream.by_ref().take(256).collect();

                    // 2.a.1.2 Ensure the collected bits are the correct length for a secp256k1 public key bytes.
                    if account_key_bits.len() != 256 {
                        return Err(RootAccountAPEDecodeError::AccountKeyBitsLengthError);
                    }

                    // 2.a.1.3 Convert the `RootAccount`'s schnorr account key bits to an even schnorr account key bytes.
                    let account_key_bytes: [u8; 32] = account_key_bits
                        .to_bytes()
                        .try_into()
                        .map_err(|_| RootAccountAPEDecodeError::AccountKeyBytesConversionError)?;

                    // 2.a.1.4 Check if the `RootAccount`'s schnorr account key is a valid secp256k1 point.
                    if account_key_bytes.to_even_point().is_none() {
                        return Err(
                            RootAccountAPEDecodeError::AccountKeyIsNotAValidSecp256k1PointError(
                                account_key_bytes,
                            ),
                        );
                    }

                    // 2.a.1.5 Check if the `RootAccount`'s account key is already registered.
                    let is_registered = {
                        // 2.a.1.5.1 Lock the `Registery Manager`.
                        let _registery_manager = registery_manager.lock().await;

                        // 2.a.1.5.2 Check if the `RootAccount`'s account key is already registered.
                        _registery_manager.is_account_registered(account_key_bytes)
                    };

                    // 2.a.1.6 If the `RootAccount`'s account key is already registered, return an error.
                    if is_registered {
                        return Err(RootAccountAPEDecodeError::AccountKeyAlreadyRegisteredError);
                    }

                    // 2.a.1.6 Return the `RootAccount`'s account key bytes.
                    account_key_bytes
                };

                // 2.a.2 Decode the bls key from the APE bitstream.
                let bls_key_bytes: [u8; 48] = {
                    // 2.a.2.1 Collect exactly 384 bits for the `RootAccount`'s bls key.
                    let bls_key_bits: BitVec = bit_stream.by_ref().take(384).collect();

                    // 2.a.2.2 Ensure the collected bits are the correct length for a bls key bytes.
                    if bls_key_bits.len() != 384 {
                        return Err(RootAccountAPEDecodeError::BlsKeyBitsLengthError);
                    }

                    // 2.a.2.3 Convert the `RootAccount`'s bls key bits to an even bls key bytes.
                    let bls_key_bytes: [u8; 48] = bls_key_bits
                        .to_bytes()
                        .try_into()
                        .map_err(|_| RootAccountAPEDecodeError::BlsKeyBytesConversionError)?;

                    // 2.a.2.4 Check if the `RootAccount`'s bls key is a valid bls key.
                    {
                        // TODO: Implement this.
                    }

                    // 2.a.2.5 Check if the BLS key is not conflicting with an already registered BLS key.
                    {
                        // 2.a.2.5.1 Lock the `Registery Manager`.
                        let _registery_manager = registery_manager.lock().await;

                        // 2.a.2.5.2 Check if the BLS key is conflicting with an already registered BLS key.
                        let is_conflicting = _registery_manager
                            .bls_key_is_conflicting_with_an_already_registered_bls_key(
                                bls_key_bytes,
                            );

                        // 2.a.2.5.3 If the BLS key is conflicting with an already registered BLS key, return an error.
                        if is_conflicting {
                            return Err(
                                RootAccountAPEDecodeError::BlsKeyConflictingWithAlreadyRegisteredBlsKeyError,
                            );
                        }
                    }

                    // 2.a.2.6 Return the `RootAccount`'s bls key bytes.
                    bls_key_bytes
                };

                // 2.a.3 Decode the flame config from the APE bitstream.
                let flame_config: Option<FMAccountFlameConfig> = {
                    // 2.a.3.1 Collect one bit to determine if the flame config is present.
                    let is_flame_config_present: bool = bit_stream
                        .by_ref()
                        .next()
                        .ok_or(RootAccountAPEDecodeError::FlameConfigPresentBitCollectError)?;

                    // 2.a.3.2 Match on the flame config presence.
                    match is_flame_config_present {
                        // 2.a.3.2.a The flame config is present.
                        true => {
                            // 2.a.3.2.a.1 Collect 16 bits to determine the flame config length.
                            let flame_config_length_bits: BitVec =
                                bit_stream.by_ref().take(16).collect();

                            // 2.a.3.2.a.2 Ensure the collected bits are the correct length for a flame config length.
                            if flame_config_length_bits.len() != 16 {
                                return Err(
                                    RootAccountAPEDecodeError::FlameConfigLengthBitsCollectError,
                                );
                            }

                            // 2.a.3.2.a.3 Convert the collected bits to a flame config length.
                            let flame_config_length_bytes: [u8; 2] = flame_config_length_bits
                                .to_bytes()
                                .try_into()
                                .map_err(|_| {
                                    RootAccountAPEDecodeError::FlameConfigLengthBytesConversionError
                                })?;

                            // 2.a.3.2.a.4 Convert the collected bits to a flame config length.
                            let flame_config_length: u16 =
                                u16::from_le_bytes(flame_config_length_bytes);

                            // 2.a.3.2.a.5 Collect the flame config bits.
                            let flame_config_bits: BitVec = bit_stream
                                .by_ref()
                                .take(flame_config_length as usize * 8)
                                .collect();

                            // 2.a.3.2.a.6 Ensure the collected bits are the correct length for a flame config.
                            if flame_config_bits.len() != flame_config_length as usize * 8 {
                                return Err(RootAccountAPEDecodeError::FlameConfigBitsCollectError);
                            }

                            // 2.a.3.2.a.7 Convert the collected bits to a flame config bytes.
                            let flame_config_bytes: Vec<u8> = flame_config_bits.to_bytes();

                            // 2.a.3.2.a.8 Convert the collected bits to a flame config.
                            let flame_config = FMAccountFlameConfig::from_bytes(
                                &flame_config_bytes,
                            )
                            .ok_or(RootAccountAPEDecodeError::FailedToDecodeFlameConfigError)?;

                            // 2.a.3.2.a.9 Return the flame config.
                            Some(flame_config)
                        }

                        // 2.a.3.2.b The flame config is not present.
                        false => None,
                    }
                };

                // 2.a.4 Decode the authentication signature from the APE bitstream.
                let authentication_signature_bytes: [u8; 64] = {
                    // 2.a.4.1 Collect exactly 512 bits for the `RootAccount`'s authentication signature.
                    let authentication_signature_bits: BitVec =
                        bit_stream.by_ref().take(512).collect();

                    // 2.a.4.2 Ensure the collected bits are the correct length for a authentication signature bytes.
                    if authentication_signature_bits.len() != 512 {
                        return Err(
                            RootAccountAPEDecodeError::AuthenticationSignatureBitsLengthError,
                        );
                    }

                    // 2.a.4.3 Convert the `RootAccount`'s authentication signature bits to an even authentication signature bytes.
                    let authentication_signature_bytes: [u8; 64] = authentication_signature_bits
                        .to_bytes()
                        .try_into()
                        .map_err(|_| {
                            RootAccountAPEDecodeError::AuthenticationSignatureBytesConversionError
                        })?;

                    // 2.a.4.4 Return the `RootAccount`'s authentication signature bytes.
                    authentication_signature_bytes
                };

                // 2.a.5 Authenticate the `RootAccount`'s authentication signature.
                {
                    // 2.a.5.1 Construct the message to authenticate the `RootAccount`'s authentication signature.
                    let message: [u8; 32] = {
                        // 2.a.5.1.1 Construct the preimage for the message to authenticate the `RootAccount`'s authentication signature.
                        let mut preimage = Vec::<u8>::with_capacity(32 + 48 + 32);

                        // 2.a.5.1.2 Extend the preimage with the `RootAccount`'s account key bytes.
                        preimage.extend(account_key_bytes);

                        // 2.a.5.1.3 Extend the preimage with the `RootAccount`'s bls key bytes.
                        preimage.extend(bls_key_bytes);

                        // 2.a.5.1.4 Get the `RootAccount`'s flame config hash.
                        let flame_config_hash: [u8; 32] = match &flame_config {
                            // 2.a.5.1.4.a The flame config is present.
                            Some(flame_config) => flame_config.hash(),

                            // 2.a.5.1.4.b The flame config is not present.
                            None => [0x00u8; 32],
                        };

                        // 2.a.5.1.5 Extend the preimage with the `RootAccount`'s flame config hash.
                        preimage.extend(flame_config_hash);

                        // 2.a.5.1.6 Hash the preimage to get the message.
                        let message = preimage
                            .hash(Some(HashTag::RootAccountBLSPublicKeyAuthenticationMessage));

                        // 2.a.5.1.7 Return the message.
                        message
                    };

                    // 2.a.5.2 Verify the `RootAccount`'s authentication signature.
                    if !schnorr::verify_xonly(
                        account_key_bytes,
                        message,
                        authentication_signature_bytes,
                        SchnorrSigningMode::Cube,
                    ) {
                        return Err(
                            RootAccountAPEDecodeError::AuthenticationSignatureVerificationFailed,
                        );
                    }
                }

                // 2.a.6 Construct the unregistered `RootAccount`.
                let unregistered_root_account = UnregisteredRootAccount::new(
                    account_key_bytes,
                    bls_key_bytes,
                    flame_config,
                    authentication_signature_bytes,
                );

                // 2.a.7 Return the unregistered `RootAccount`.
                let root_account = RootAccount::UnregisteredRootAccount(unregistered_root_account);

                // 2.a.8 Return the unregistered `RootAccount`.
                Ok(root_account)
            }

            // 2.b The `RootAccount` is registered.
            _ => {
                // 2.b.1 Get account key and BLS key by rank.
                let (account_key, bls_key): ([u8; 32], Option<[u8; 48]>) = {
                    // 2.b.1.1 Lock the `Registery Manager`.
                    let _registery_manager = registery_manager.lock().await;

                    // 2.b.1.2 Get RMAccountBody by rank.
                    _registery_manager
                        .get_account_key_and_bls_key_by_rank(rank)
                        .ok_or(
                            RootAccountAPEDecodeError::FailedToRetrieveRMAccountBodyByRank(rank),
                        )?
                };

                // 2.b.2 Match on whether the BLS key is configured or not.
                match bls_key {
                    // 2.b.2.a The BLS key is configured.
                    Some(bls_key) => {
                        // 2.b.2.a.1 Construct the `RegisteredAndConfiguredRootAccount`.
                        let registered_and_configured_root_account =
                            RegisteredAndConfiguredRootAccount::new(account_key, bls_key);

                        // 2.b.2.a.2 Construct the `RootAccount`.
                        let root_account = RootAccount::RegisteredAndConfiguredRootAccount(
                            registered_and_configured_root_account,
                        );

                        // 2.b.2.a.3 Return the `RootAccount`.
                        Ok(root_account)
                    }

                    // 2.b.2.b The BLS key is not configured.
                    None => {
                        // 2.b.2.b.1 Decode the bls key from the APE bitstream.
                        let bls_key_bytes: [u8; 48] = {
                            // 2.b.2.b.1.1 Collect exactly 384 bits for the `RootAccount`'s bls key.
                            let bls_key_bits: BitVec = bit_stream.by_ref().take(384).collect();

                            // 2.b.2.b.1.2 Ensure the collected bits are the correct length for a bls key bytes.
                            if bls_key_bits.len() != 384 {
                                return Err(RootAccountAPEDecodeError::BlsKeyBitsLengthError);
                            }

                            // 2.b.2.b.1.3 Convert the `RootAccount`'s bls key bits to an even bls key bytes.
                            let bls_key_bytes: [u8; 48] =
                                bls_key_bits.to_bytes().try_into().map_err(|_| {
                                    RootAccountAPEDecodeError::BlsKeyBytesConversionError
                                })?;

                            // 2.b.2.b.1.4 Check if the `RootAccount`'s bls key is a valid bls key.
                            {
                                // TODO: Implement this.
                            }

                            // 2.b.2.b.1.5 Check if the BLS key is not conflicting with an already registered BLS key.
                            {
                                // 2.b.2.b.1.5.1 Lock the `Registery Manager`.
                                let _registery_manager = registery_manager.lock().await;

                                // 2.b.2.b.1.5.2 Check if the BLS key is conflicting with an already registered BLS key.
                                let is_conflicting = _registery_manager
                                    .bls_key_is_conflicting_with_an_already_registered_bls_key(
                                        bls_key_bytes,
                                    );

                                // 2.b.2.b.1.5.3 If the BLS key is conflicting with an already registered BLS key, return an error.
                                if is_conflicting {
                                    return Err(
                                RootAccountAPEDecodeError::BlsKeyConflictingWithAlreadyRegisteredBlsKeyError,
                            );
                                }
                            }

                            // 2.b.2.b.1.6 Return the `RootAccount`'s bls key bytes.
                            bls_key_bytes
                        };

                        // 2.b.2.b.2 Decode the flame config from the APE bitstream.
                        let flame_config: Option<FMAccountFlameConfig> = {
                            // 2.b.2.b.2.1 Collect one bit to determine if the flame config is present.
                            let is_flame_config_present: bool = bit_stream.by_ref().next().ok_or(
                                RootAccountAPEDecodeError::FlameConfigPresentBitCollectError,
                            )?;

                            // 2.b.2.b.2.2 Match on the flame config presence.
                            match is_flame_config_present {
                                // 2.b.2.b.2.2.a The flame config is present.
                                true => {
                                    // 2.b.2.b.2.2.a.1 Collect 16 bits to determine the flame config length.
                                    let flame_config_length_bits: BitVec =
                                        bit_stream.by_ref().take(16).collect();

                                    // 2.b.2.b.2.2.a.2 Ensure the collected bits are the correct length for a flame config length.
                                    if flame_config_length_bits.len() != 16 {
                                        return Err(
                                    RootAccountAPEDecodeError::FlameConfigLengthBitsCollectError,
                                );
                                    }

                                    // 2.b.2.b.2.2.a.3 Convert the collected bits to a flame config length.
                                    let flame_config_length_bytes: [u8; 2] = flame_config_length_bits
                                .to_bytes()
                                .try_into()
                                .map_err(|_| {
                                    RootAccountAPEDecodeError::FlameConfigLengthBytesConversionError
                                })?;

                                    // 2.b.2.b.2.2.a.4 Convert the collected bits to a flame config length.
                                    let flame_config_length: u16 =
                                        u16::from_le_bytes(flame_config_length_bytes);

                                    // 2.b.2.b.2.2.a.5 Collect the flame config bits.
                                    let flame_config_bits: BitVec = bit_stream
                                        .by_ref()
                                        .take(flame_config_length as usize * 8)
                                        .collect();

                                    // 2.b.2.b.2.2.a.6 Ensure the collected bits are the correct length for a flame config.
                                    if flame_config_bits.len() != flame_config_length as usize * 8 {
                                        return Err(
                                            RootAccountAPEDecodeError::FlameConfigBitsCollectError,
                                        );
                                    }

                                    // 2.b.2.b.2.2.a.7 Convert the collected bits to a flame config bytes.
                                    let flame_config_bytes: Vec<u8> = flame_config_bits.to_bytes();

                                    // 2.b.2.b.2.2.a.8 Convert the collected bits to a flame config.
                                    let flame_config = FMAccountFlameConfig::from_bytes(
                                        &flame_config_bytes,
                                    )
                                    .ok_or(
                                        RootAccountAPEDecodeError::FailedToDecodeFlameConfigError,
                                    )?;

                                    // 2.b.2.b.2.2.a.9 Return the flame config.
                                    Some(flame_config)
                                }

                                // 2.b.2.b.2.2.b The flame config is not present.
                                false => None,
                            }
                        };

                        // 2.b.2.b.3 Decode the authentication signature from the APE bitstream.
                        let authentication_signature_bytes: [u8; 64] = {
                            // 2.b.2.b.3.1 Collect exactly 512 bits for the `RootAccount`'s authentication signature.
                            let authentication_signature_bits: BitVec =
                                bit_stream.by_ref().take(512).collect();

                            // 2.b.2.b.3.2 Ensure the collected bits are the correct length for a authentication signature bytes.
                            if authentication_signature_bits.len() != 512 {
                                return Err(
                            RootAccountAPEDecodeError::AuthenticationSignatureBitsLengthError,
                        );
                            }

                            // 2.b.2.b.3.3 Convert the `RootAccount`'s authentication signature bits to an even authentication signature bytes.
                            let authentication_signature_bytes: [u8; 64] = authentication_signature_bits
                        .to_bytes()
                        .try_into()
                        .map_err(|_| {
                            RootAccountAPEDecodeError::AuthenticationSignatureBytesConversionError
                        })?;

                            // 2.b.2.b.3.4 Return the `RootAccount`'s authentication signature bytes.
                            authentication_signature_bytes
                        };

                        // 2.b.2.b.4 Authenticate the `RootAccount`'s authentication signature.
                        {
                            // 2.b.2.b.4.1 Construct the message to authenticate the `RootAccount`'s authentication signature.
                            let message: [u8; 32] = {
                                // 2.b.2.b.4.1.1 Construct the preimage for the message to authenticate the `RootAccount`'s authentication signature.
                                let mut preimage = Vec::<u8>::with_capacity(32 + 48 + 32);

                                // 2.b.2.b.4.1.2 Extend the preimage with the `RootAccount`'s account key bytes.
                                preimage.extend(account_key);

                                // 2.b.2.b.4.1.3 Extend the preimage with the `RootAccount`'s bls key bytes.
                                preimage.extend(bls_key_bytes);

                                // 2.b.2.b.4.1.4 Get the `RootAccount`'s flame config hash.
                                let flame_config_hash: [u8; 32] = match &flame_config {
                                    // 2.b.2.b.4.1.4.a The flame config is present.
                                    Some(flame_config) => flame_config.hash(),

                                    // 2.b.2.b.4.1.4.b The flame config is not present.
                                    None => [0x00u8; 32],
                                };

                                // 2.b.2.b.4.1.5 Extend the preimage with the `RootAccount`'s flame config hash.
                                preimage.extend(flame_config_hash);

                                // 2.b.2.b.4.1.6 Hash the preimage to get the message.
                                let message = preimage.hash(Some(
                                    HashTag::RootAccountBLSPublicKeyAuthenticationMessage,
                                ));

                                // 2.b.2.b.4.1.7 Return the message.
                                message
                            };

                            // 2.b.2.b.4.2 Verify the `RootAccount`'s authentication signature.
                            if !schnorr::verify_xonly(
                                account_key,
                                message,
                                authentication_signature_bytes,
                                SchnorrSigningMode::Cube,
                            ) {
                                return Err(
                            RootAccountAPEDecodeError::AuthenticationSignatureVerificationFailed,
                        );
                            }
                        }

                        // 2.b.2.b.5 Construct the registered but unconfigured `RootAccount`.
                        let registered_but_unconfigured_root_account =
                            RegisteredButUnconfiguredRootAccount::new(
                                account_key,
                                bls_key_bytes,
                                flame_config,
                                authentication_signature_bytes,
                            );

                        // 2.b.2.b.6 Construct the `RootAccount`.
                        let root_account = RootAccount::RegisteredButUnconfiguredRootAccount(
                            registered_but_unconfigured_root_account,
                        );

                        // 2.b.2.b.7 Return the `RootAccount`.
                        return Ok(root_account);
                    }
                }
            }
        }
    }
}
