use crate::constructive::entity::contract::ape::decode::error::decode_error::ContractAPEDecodeError;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registry::registry::REGISTRY;

impl Contract {
    /// Decodes a `Contract` from an Airly Payload Encoding (APE) bit vector.
    ///
    /// The rank value is decoded as a `LongVal` or a `ShortVal`, then resolved via the registry.
    pub async fn decode_ape<'a>(
        bit_stream: &mut bit_vec::Iter<'a>,
        registry: &REGISTRY,
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

        // 2 Retrieve the `Contract` from the registry by its rank.
        let cube = {
            let _registry = registry.lock().await;

            _registry.get_contract_by_rank(rank).ok_or(
                ContractAPEDecodeError::ContractNotFoundInRegistryWithRank(rank),
            )?
        };

        Ok(cube)
    }
}
