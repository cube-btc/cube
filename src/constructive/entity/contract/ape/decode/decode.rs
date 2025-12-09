use crate::constructive::entity::contract::ape::decode::error::decode_error::ContractAPEDecodeError;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;

impl Contract {
    /// Decodes a `Contract` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function decodes a `Contract` as an Airly Payload Encoding (APE) bit vector.
    /// The `Contract` can be either deployed or undeployed.
    /// If the `Contract` is deployed, the rank value is decoded as a `LongVal` or a `ShortVal`.
    /// If the `Contract` is undeployed, the public key is decoded as a bit vector.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `registery_manager` - The `Registery Manager`.
    /// * `decode_rank_as_longval` - Whether to decode the rank value as a `LongVal` or a `ShortVal`.
    pub async fn decode_ape<'a>(
        bit_stream: &mut bit_vec::Iter<'a>,
        registery_manager: &REGISTERY_MANAGER,
        decode_rank_as_longval: bool,
    ) -> Result<Contract, ContractAPEDecodeError> {
        // 1 Match on whether to decode the rank value as a `LongVal` or a `ShortVal`.
        let rank: u64 = match decode_rank_as_longval {
            // 1.a The rank is decoded as a `LongVal`.
            true => {
                // 1.a.1 Decode the rank value as a `LongVal`.
                let rank = LongVal::decode_ape(bit_stream)
                    .map_err(|e| ContractAPEDecodeError::FailedToDecodeRankValueAsLongVal(e))?
                    .value();

                // 1.a.2 Return the rank value as a `u64`.
                rank
            }

            // 1.b The rank is decoded as a `ShortVal`.
            false => {
                // 1.b.1 Decode the rank value as a `ShortVal`.
                let rank = ShortVal::decode_ape(bit_stream)
                    .map_err(|e| ContractAPEDecodeError::FailedToDecodeRankValueAsShortVal(e))?
                    .value();

                // 1.b.2 Return the rank value as a `u64`.
                rank as u64
            }
        };

        // 2 Retrieve the `Contract` from the `Registery Manager` by its rank.
        let contract = {
            // 2.a Lock the `Registery Manager`.
            let _registery_manager = registery_manager.lock().await;

            // 2.b Retrieve the `Contract` from the `Registery Manager` by its rank.
            _registery_manager
                .get_contract_by_rank(rank)
                .ok_or(ContractAPEDecodeError::ContractNotFoundInRegisteryManagerWithRank(rank))?
        };

        // 3 Return the decoded `Contract`.
        return Ok(contract);
    }
}
