use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::account::ape::decode::error::decode_error::AccountAPEDecodeError;
use crate::constructive::entity::account::account::unregistered_account::unregistered_account::UnregisteredAccount;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use bit_vec::BitVec;
use secp::Point;

impl Account {
    /// Decodes an `Account` as an Airly Payload Encoding (APE) bit vector.  
    ///
    /// This function decodes an `Account` as an Airly Payload Encoding (APE) bit vector.
    /// The `Account` can be either registered or unregistered.
    /// If the `Account` is registered, the rank value is decoded as a `LongVal` or a `ShortVal`.
    /// If the `Account` is unregistered, the public key is decoded as a bit vector.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `registery_manager` - The `Registery Manager`.
    /// * `decode_rank_as_longval` - Whether to decode the rank value as a `LongVal` or a `ShortVal`.
    pub async fn decode_ape<'a>(
        bit_stream: &mut bit_vec::Iter<'a>,
        registery_manager: &REGISTERY_MANAGER,
        decode_rank_as_longval: bool,
    ) -> Result<Account, AccountAPEDecodeError> {
        // 1 Decode the rank value from the APE bitstream.
        let rank: u64 = match decode_rank_as_longval {
            // 1.a The rank is decoded as a `LongVal`.
            true => LongVal::decode_ape(bit_stream)
                .map_err(|e| AccountAPEDecodeError::FailedToDecodeRankValueAsLongVal(e))?
                .value(),

            // 1.b The rank is decoded as a `ShortVal`.
            false => ShortVal::decode_ape(bit_stream)
                .map_err(|e| AccountAPEDecodeError::FailedToDecodeRankValueAsShortVal(e))?
                .value() as u64,
        };

        // 2 Match the rank value to determine if the `Account` is registered or not.
        // If rank is zero, then we interpret this as an unregistered account, otherwise it is a registered account.
        match rank {
            // 2.a The `Account` is unregistered.
            0 => {
                // 2.a.1 Collect exactly 256 bits for the `Account`'s public key.
                let public_key_bits: BitVec = bit_stream.by_ref().take(256).collect();

                // 2.a.2 Ensure the collected bits are the correct length for a secp256k1 public key bytes.
                if public_key_bits.len() != 256 {
                    return Err(AccountAPEDecodeError::PublicKeyBitsLengthError);
                }

                // 2.a.3 Convert the `Account`'s public key bits to an even public key bytes.
                let mut public_key_bytes = vec![0x02];
                public_key_bytes.extend(public_key_bits.to_bytes());

                // 2.a.4 Check if the `Account`'s public key is a valid secp256k1 point.
                let public_key_point = Point::from_slice(&public_key_bytes)
                    .map_err(|_| AccountAPEDecodeError::PublicKeyPointFromSliceError)?;

                // 2.a.5 Serialize the `Account`'s public key point to x-only bytes.
                let public_key = public_key_point.serialize_xonly();

                // 2.a.6 Check if the `Account`'s key is already registered.
                let is_registered = {
                    let _registery_manager = registery_manager.lock().await;
                    _registery_manager.is_account_registered(public_key)
                };

                // 2.a.7 If the `Account`'s key is already registered, return an error.
                if is_registered {
                    return Err(AccountAPEDecodeError::KeyAlreadyRegisteredError);
                }

                // 2.a.8 Construct the unregistered `Account`.
                let unregistered_account = UnregisteredAccount::new(public_key);

                // 2.a.9 Return the unregistered `Account`.
                let account = Account::UnregisteredAccount(unregistered_account);

                // 2.a.10 Return the unregistered `Account`.
                return Ok(account);
            }

            // 2.b The `Account` is registered.
            _ => {
                // 2.b.1 Retrieve the `Account` from the `Registery Manager` by its rank.
                let account = {
                    // 2.b.1.1 Lock the `Registery Manager`.
                    let _registery_manager = registery_manager.lock().await;

                    // 2.b.1.2 Retrieve the `Account` from the `Registery Manager` by its rank.
                    _registery_manager
                        .get_account_by_rank(rank)
                        .ok_or(AccountAPEDecodeError::FailedToLocateAccountGivenRank(rank))?
                };

                // 2.b.2 Return the `Account`.
                return Ok(account);
            }
        }
    }
}
