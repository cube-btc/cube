use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::entity::contract::cpe::decode::decode_error::ContractCPEDecodingError;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery::contract_registery::contract_registery::CONTRACT_REGISTERY;

impl Contract {
    /// Compact payload decoding for `Contract`.
    /// Decodes a `Contract` from a bit stream.
    pub async fn decode_cpe<'a>(
        bit_stream: &mut bit_vec::Iter<'a>,
        contract_registery: &CONTRACT_REGISTERY,
    ) -> Result<Contract, ContractCPEDecodingError> {
        // Decode the rank value.
        let rank = ShortVal::decode_cpe(bit_stream)
            .map_err(|e| ContractCPEDecodingError::RankAsShortValDecodeError(e))?
            .value();

        // Retrieve the contract given rank value.
        let contract = {
            let _contract_registery = contract_registery.lock().await;
            _contract_registery.get_contract_by_rank(rank).ok_or(
                ContractCPEDecodingError::FailedToLocateContractGivenRank(rank),
            )?
        };

        // Return the contract.
        return Ok(contract);
    }
}
