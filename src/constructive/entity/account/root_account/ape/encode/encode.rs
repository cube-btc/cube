use crate::constructive::entity::account::root_account::ape::encode::error::encode_error::RootAccountAPEEncodeError;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use bit_vec::BitVec;

impl RootAccount {
    /// Encodes a `RootAccount` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function encodes a `RootAccount` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// # Arguments
    /// * `&self` - The `RootAccount` to encode.
    /// * `registery_manager` - The guarded `RegisteryManager` to get the `Account`'s rank value.
    /// * `encode_rank_as_longval` - Whether to encode the `RootAccount`'s rank value as a `LongVal` or a `ShortVal`.
    pub async fn encode_ape(
        &self,
        registery_manager: &REGISTERY_MANAGER,
        encode_rank_as_longval: bool,
    ) -> Result<BitVec, RootAccountAPEEncodeError> {
        // 1 Initialize the APE bit vector.
        let mut bits = BitVec::new();

        // 2 Match on the `RootAccount` variant.
        match self {
            // 2.a Encode the `RootAccount` as a `UnregisteredRootAccount`.
            RootAccount::UnregisteredRootAccount(unregistered_root_account) => {
                // 2.a.1 Encode the account key.
                {
                    // 2.a.1.1 Get the account key bytes to encode.
                    let account_key_bytes_to_encode =
                        unregistered_root_account.account_key_to_be_registered;

                    // 2.a.1.2 Convert the account key bytes to bits.
                    let account_key_bits = BitVec::from_bytes(&account_key_bytes_to_encode);

                    // 2.a.1.3 Extend the APE bit vector with the account key bits.
                    bits.extend(account_key_bits);
                }

                // 2.a.2 Encode the bls key.
                {
                    // 2.a.2.1 Get the bls key bytes to encode.
                    let bls_key_bytes_to_encode =
                        unregistered_root_account.bls_key_to_be_configured;

                    // 2.a.2.2 Convert the bls key bytes to bits.
                    let bls_key_bits = BitVec::from_bytes(&bls_key_bytes_to_encode);

                    // 2.a.2.3 Extend the APE bit vector with the bls key bits.
                    bits.extend(bls_key_bits);
                }

                // 2.a.3 Encode the flame config if there is.
                match &unregistered_root_account.flame_config_to_be_configured {
                    // 2.a.3.a The flame config is present.
                    Some(flame_config) => {
                        // 2.a.3.a.1 Push a true bit to indicate that the flame config is present.
                        bits.push(true);

                        // 2.a.3.a.2 Serialize the flame config to bytes.
                        let flame_config_bytes = flame_config.to_bytes();

                        // 2.a.3.a.3 Convert the flame config bytes to bits.
                        let flame_config_bits = BitVec::from_bytes(&flame_config_bytes);

                        // 2.a.3.a.4 Get the flame config length bytes.
                        let flame_config_len: [u8; 2] =
                            (flame_config_bytes.len() as u16).to_le_bytes();

                        // 2.a.3.a.5 Convert the flame config length bytes to bits.
                        let flame_config_len_bits = BitVec::from_bytes(&flame_config_len);

                        // 2.a.3.a.6 Push the flame config length bits.
                        bits.extend(flame_config_len_bits);

                        // 2.a.3.a.7 Push the flame config bits.
                        bits.extend(flame_config_bits);
                    }
                    None => {
                        // 2.a.3.b Push a false bit to indicate that the flame config is not present.
                        bits.push(false);
                    }
                }

                // 2.a.4 Encode the authentication signature.
                {
                    // 2.a.4.1 Get the authentication signature bytes to encode.
                    let authentication_signature_bytes_to_encode =
                        unregistered_root_account.authentication_signature;

                    // 2.a.4.2 Convert the authentication signature bytes to bits.
                    let authentication_signature_bits =
                        BitVec::from_bytes(&authentication_signature_bytes_to_encode);

                    // 2.a.4.3 Extend the APE bit vector with the authentication signature bits.
                    bits.extend(authentication_signature_bits);
                }
            }
            // 2.b Encode the `RootAccount` as a `RegisteredButBLSUnconfiguredRootAccount`.
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 2.b.1 Encode the rank value.
                {
                    // 2.b.1.1 Retrieve the rank value from the `RegisteryManager`.
                    let rank = {
                        // 2.b.1.1.1 Lock the `RegisteryManager`.
                        let _registery_manager = registery_manager.lock().await;

                        // 2.b.1.1.2 Retrieve the rank value from the `RegisteryManager`.
                        _registery_manager.get_rank_by_account_key(registered_but_unconfigured_root_account.account_key).ok_or(
                        RootAccountAPEEncodeError::UnableToRetrieveRankValueFromRegisteryManager(
                            registered_but_unconfigured_root_account.account_key,
                        ),
                    )?
                    };

                    // 2.b.1.2 Match on whether to encode the rank value as a `LongVal` or a `ShortVal`.
                    match encode_rank_as_longval {
                        // 2.b.2.a The rank is to be encoded as a `LongVal`.
                        true => {
                            // 2.b.2.a.1 Convert the rank value into a `LongVal`.
                            let rank_as_longval = LongVal::new(rank);

                            // 2.b.2.a.2 Extend the APE bit vector with the rank value as a `LongVal`.
                            bits.extend(rank_as_longval.encode_ape());
                        }

                        // 2.b.2.b The rank is to be encoded as a `ShortVal`.
                        false => {
                            // 2.b.2.b.1 Convert the rank value into a `ShortVal`.
                            let rank_as_shortval = ShortVal::new(rank as u32);

                            // 2.b.2.b.2 Extend the APE bit vector with the rank value as a `ShortVal`.
                            bits.extend(rank_as_shortval.encode_ape());
                        }
                    }
                }

                // 2.b.2 Encode the bls key.
                {
                    // 2.b.2.1 Get the bls key bytes to encode.
                    let bls_key_bytes_to_encode =
                        registered_but_unconfigured_root_account.bls_key_to_be_configured;

                    // 2.b.2.2 Convert the bls key bytes to bits.
                    let bls_key_bits = BitVec::from_bytes(&bls_key_bytes_to_encode);

                    // 2.b.2.3 Extend the APE bit vector with the bls key bits.
                    bits.extend(bls_key_bits);
                }

                // 2.b.3 Encode the flame config if there is.
                match &registered_but_unconfigured_root_account.flame_config_to_be_configured {
                    // 2.b.3.a The flame config is present.
                    Some(flame_config) => {
                        // 2.b.3.a.1 Push a true bit to indicate that the flame config is present.
                        bits.push(true);

                        // 2.b.3.a.2 Serialize the flame config to bytes.
                        let flame_config_bytes = flame_config.to_bytes();

                        // 2.b.3.a.3 Convert the flame config bytes to bits.
                        let flame_config_bits = BitVec::from_bytes(&flame_config_bytes);

                        // 2.b.3.a.4 Get the flame config length bytes.
                        let flame_config_len: [u8; 2] =
                            (flame_config_bytes.len() as u16).to_le_bytes();

                        // 2.b.3.a.5 Convert the flame config length bytes to bits.
                        let flame_config_len_bits = BitVec::from_bytes(&flame_config_len);

                        // 2.b.3.a.6 Push the flame config length bits.
                        bits.extend(flame_config_len_bits);

                        // 2.b.3.a.7 Push the flame config bits.
                        bits.extend(flame_config_bits);
                    }
                    None => {
                        // 2.b.3.b Push a false bit to indicate that the flame config is not present.
                        bits.push(false);
                    }
                }

                // 2.b.4 Encode the authentication signature.
                {
                    // 2.b.4.1 Get the authentication signature bytes to encode.
                    let authentication_signature_bytes_to_encode =
                        registered_but_unconfigured_root_account.authentication_signature;

                    // 2.b.4.2 Convert the authentication signature bytes to bits.
                    let authentication_signature_bits =
                        BitVec::from_bytes(&authentication_signature_bytes_to_encode);

                    // 2.b.4.3 Extend the APE bit vector with the authentication signature bits.
                    bits.extend(authentication_signature_bits);
                }
            }
            // 2.c Encode the `RootAccount` as a `RegisteredAndBLSConfiguredRootAccount`.
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 2.c.1 Encode the rank value.
                {
                    // 2.c.1.1 Retrieve the rank value from the `RegisteryManager`.
                    let rank = {
                        // 2.c.1.1.1 Lock the `RegisteryManager`.
                        let _registery_manager = registery_manager.lock().await;

                        // 2.c.1.1.2 Retrieve the rank value from the `RegisteryManager`.
                        _registery_manager.get_rank_by_account_key(registered_and_configured_root_account.account_key).ok_or(
                        RootAccountAPEEncodeError::UnableToRetrieveRankValueFromRegisteryManager(
                            registered_and_configured_root_account.account_key,
                        ),
                    )?
                    };

                    // 2.c.1.2 Match on whether to encode the rank value as a `LongVal` or a `ShortVal`.
                    match encode_rank_as_longval {
                        // 2.c.1.2.a The rank is to be encoded as a `LongVal`.
                        true => {
                            // 2.c.1.2.a.1 Convert the rank value into a `LongVal`.
                            let rank_as_longval = LongVal::new(rank);

                            // 2.c.1.2.a.2 Extend the APE bit vector with the rank value as a `LongVal`.
                            bits.extend(rank_as_longval.encode_ape());
                        }

                        // 2.c.1.2.b The rank is to be encoded as a `ShortVal`.
                        false => {
                            // 2.c.1.2.b.1 Convert the rank value into a `ShortVal`.
                            let rank_as_shortval = ShortVal::new(rank as u32);

                            // 2.c.1.2.b.2 Extend the APE bit vector with the rank value as a `ShortVal`.
                            bits.extend(rank_as_shortval.encode_ape());
                        }
                    }
                }
            }
        }

        // 3 Return the encoded bit vector.
        Ok(bits)
    }
}
