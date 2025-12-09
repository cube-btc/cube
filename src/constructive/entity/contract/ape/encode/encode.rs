use crate::constructive::{
    entity::contract::{
        ape::encode::error::encode_error::ContractAPEEncodeError, contract::Contract,
    },
    valtype::val::{long_val::long_val::LongVal, short_val::short_val::ShortVal},
};
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
    /// * `encode_rank_as_longval` - Whether to encode the `Contract`'s rank value as a `LongVal` or a `ShortVal`.
    pub fn encode_ape(
        &self,
        encode_rank_as_longval: bool,
    ) -> Result<BitVec, ContractAPEEncodeError> {
        // 1 Initialize the bit vector.
        let mut bits = BitVec::new();

        // 2 Match on whether the `Contract` is a deployed `Contract` or an undeployed `Contract`.
        let rank: u64 = match self {
            // 2.a The `Contract` is a deployed `Contract`.
            Contract::DeployedContract(deployed_contract) => {
                // 2.a.1 Get the rank value.
                match deployed_contract.rank {
                    // 2.a.1.a The `Contract` has its rank value set.
                    Some(rank) => rank,

                    // 2.a.1.b The `Contract` has its rank value not set.
                    None => {
                        // 2.a.1.b.1 Return an error if the `Contract` has its rank value not set.
                        return Err(ContractAPEEncodeError::RankNotFoundError(
                            self.contract_id(),
                        ));
                    }
                }
            }

            // 2.b The `Contract` is an undeployed `Contract`.
            Contract::UndeployedContract(_) => {
                // 2.b.1 Return an error if the `Contract` is a non-deployed `Contract`, because it cannot be APE-encoded.
                return Err(
                    ContractAPEEncodeError::UndeployedContractCannotBeEncodedError(
                        self.contract_id(),
                    ),
                );
            }
        };

        // 3 Match on whether to encode the rank value as a `LongVal` or a `ShortVal`.
        match encode_rank_as_longval {
            // 3.a The rank is to be encoded as a `LongVal`.
            true => {
                // 3.a.1 Convert the rank value into a `LongVal`.
                let rank_as_longval = LongVal::new(rank);

                // 3.a.2 Extend the rank value with the `LongVal`.
                bits.extend(rank_as_longval.encode_ape());
            }

            // 3.b The rank is to be encoded as a `ShortVal`.
            false => {
                // 3.b.1 Convert the rank value into a `ShortVal`.
                let rank_as_shortval = ShortVal::new(rank as u32);

                // 3.b.2 Extend the rank value with the `ShortVal`.
                bits.extend(rank_as_shortval.encode_ape());
            }
        }

        // 4 Return the encoded bit vector.
        Ok(bits)
    }
}
