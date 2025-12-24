use crate::constructive::{
    entity::contract::{
        ape::encode::error::encode_error::ContractAPEEncodeError, contract::Contract,
    },
    valtype::val::{long_val::long_val::LongVal, short_val::short_val::ShortVal},
};
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
use bit_vec::BitVec;

impl Contract {
    /// Encodes a `Contract` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function encodes a `Contract` as an Airly Payload Encoding (APE) bit vector.
    /// The `Contract` can be either deployed or undeployed.
    /// If the `Contract` is deployed, the rank value is encoded as a `LongVal` or a `ShortVal`.
    /// If the `Contract` is undeployed, the public key is encoded as a bit vector.
    ///
    /// # Arguments
    /// * `&self` - The `Contract` to encode.
    /// * `registery_manager` - The guarded `RegisteryManager` to get the `Contract`'s rank value.
    /// * `encode_rank_as_longval` - Whether to encode the `Contract`'s rank value as a `LongVal` or a `ShortVal`.
    pub async fn encode_ape(
        &self,
        registery_manager: &REGISTERY_MANAGER,
        encode_rank_as_longval: bool,
    ) -> Result<BitVec, ContractAPEEncodeError> {
        // 1 Initialize the bit vector.
        let mut bits = BitVec::new();

        // 2 Get the contract id.
        let contract_id = self.contract_id();

        // 3 Retrieve the rank value from the registery manager.
        let rank: u64 = {
            // 3.1 Lock the registery manager.
            let _registery_manager = registery_manager.lock().await;

            // 3.2 Retrieve the rank value from the registery manager.
            _registery_manager
                .get_rank_by_contract_id(contract_id)
                .ok_or(
                    ContractAPEEncodeError::UnableToRetrieveRankValueFromRegisteryManager(
                        contract_id,
                    ),
                )?
        };

        // 4 Match on whether to encode the rank value as a `LongVal` or a `ShortVal`.
        match encode_rank_as_longval {
            // 4.a The rank is to be encoded as a `LongVal`.
            true => {
                // 4.a.1 Convert the rank value into a `LongVal`.
                let rank_as_longval = LongVal::new(rank);

                // 4.a.2 Extend the rank value with the `LongVal`.
                bits.extend(rank_as_longval.encode_ape());
            }

            // 4.b The rank is to be encoded as a `ShortVal`.
            false => {
                // 4.b.1 Convert the rank value into a `ShortVal`.
                let rank_as_shortval = ShortVal::new(rank as u32);

                // 4.b.2 Extend the rank value with the `ShortVal`.
                bits.extend(rank_as_shortval.encode_ape());
            }
        }

        // 5 Return the encoded bit vector.
        Ok(bits)
    }
}
