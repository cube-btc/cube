use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::account::ape::encode::error::encode_error::AccountAPEEncodeError;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use bit_vec::BitVec;

impl Account {
    /// Encodes an `Account` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function encodes an `Account` as an Airly Payload Encoding (APE) bit vector.
    /// The `Account` can be either registered or unregistered.
    /// If the `Account` is registered, the rank value is encoded as a `LongVal` or a `ShortVal`.
    /// If the `Account` is unregistered, the public key is encoded as a bit vector.
    ///
    /// # Arguments
    /// * `&self` - The `Account` to encode.
    /// * `registery_manager` - The guarded `RegisteryManager` to get the `Account`'s rank value.
    /// * `encode_rank_as_longval` - Whether to encode the `Account`'s rank value as a `LongVal` or a `ShortVal`.
    pub async fn encode_ape(
        &self,
        registery_manager: &REGISTERY_MANAGER,
        encode_rank_as_longval: bool,
    ) -> Result<BitVec, AccountAPEEncodeError> {
        // 1 Initialize the APE bit vector.
        let mut bits = BitVec::new();

        // 2 Match on whether the `Account` is registered or not.
        match self.is_registered() {
            // 2.a The `Account` is registered.
            true => {
                //
                // When the `Account` is registered, we encode the `Account`'s rank value as a `LongVal` or a `ShortVal`.
                // If the `Account` has a rank, we encode the rank value as a `LongVal` or a `ShortVal` to indicate that the `Account` is registered.
                // If the `Account` has no rank, we return an error because it is not a valid registered `Account`.
                //

                // 2.a.1 Get the rank value.
                let rank = {
                    // 2.a.1.1 Lock the `Registery Manager`.
                    let _registery_manager = registery_manager.lock().await;

                    // 2.a.1.2 Get the `Account`'s rank value from the `Registery Manager`.
                    _registery_manager
                        .get_rank_by_account_key(self.account_key())
                        .ok_or(
                            AccountAPEEncodeError::UnableToRetrieveRankValueFromRegisteryManager(
                                self.account_key(),
                            ),
                        )?
                };

                // 2.a.2 Match on whether to encode the rank value as a `LongVal` or a `ShortVal`.
                match encode_rank_as_longval {
                    // 2.a.2.a The rank is to be encoded as a `LongVal`.
                    true => {
                        // 2.a.2.a.1 Convert the rank value into a `LongVal`.
                        let rank_as_longval = LongVal::new(rank);

                        // 2.a.2.a.2 Extend the rank value with the `LongVal`.
                        bits.extend(rank_as_longval.encode_ape());
                    }

                    // 2.a.2.b The rank is to be encoded as a `ShortVal`.
                    false => {
                        // 2.a.2.b.1 Convert the rank value into a `ShortVal`.
                        let rank_as_shortval = ShortVal::new(rank as u32);

                        // 2.a.2.b.2 Extend the rank value with the `ShortVal`.
                        bits.extend(rank_as_shortval.encode_ape());
                    }
                }
            }

            // 2.b The `Account` is not registered.
            false => {
                //
                // When the `Account` is not registered, we encode a zero rank value as a `LongVal` or a `ShortVal` to indicate that the `Account` is unregistered.
                // and right afterwards we encode the full 256 public key bits of the `Account`'s public key to register it with the `RegisteryManager`.
                //

                // 2.b.1 Match on whether to encode the zero rank value as a `LongVal` or a `ShortVal`.
                match encode_rank_as_longval {
                    // 2.b.1.a The zero rank value is to be encoded as a `LongVal`.
                    true => {
                        // 2.b.1.a.1 Construct a zero `LongVal` for the zero rank value.
                        let zero_as_longval = LongVal::new(0);

                        // 2.b.1.a.2 Extend the APE bit vector with the zero `LongVal` for the zero rank value.
                        bits.extend(zero_as_longval.encode_ape());
                    }

                    // 2.b.1.b The zero rank value is to be encoded as a `ShortVal`.
                    false => {
                        // 2.b.1.b.1 Construct a zero `ShortVal` for the zero rank value.
                        let zero_as_shortval = ShortVal::new(0);

                        // 2.b.1.b.2 Extend the APE bit vector with the zero `ShortVal` for the zero rank value.
                        bits.extend(zero_as_shortval.encode_ape());
                    }
                }

                // 2.b.2 Get the `Account`'s public key bytes.
                let public_key = self.account_key();

                // 2.b.3 Get the `Account`'s public key bits.
                let public_key_bits = BitVec::from_bytes(&public_key);

                // 2.b.4 Extend the APE bit vector with the `Account`'s public key bits.
                bits.extend(public_key_bits);
            }
        }

        // 3 Return the encoded bit vector.
        Ok(bits)
    }
}
