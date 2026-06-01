use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::entity::contract::ext::codec::sbe::decode::error::decode_error::ContractSBEDecodeError;
use crate::constructive::entity::contract::ext::codec::sbe::encode::encode::CONTRACT_SBE_LEN;

impl Contract {
    /// Decodes a `Contract` from SBE bytes produced by [`Contract::encode_sbe`].
    pub fn decode_sbe(bytes: &[u8]) -> Result<Contract, ContractSBEDecodeError> {
        if bytes.len() != CONTRACT_SBE_LEN {
            return Err(ContractSBEDecodeError::ContractSBEInvalidPayloadLength {
                got: bytes.len(),
                expected: CONTRACT_SBE_LEN,
            });
        }

        let contract_id: [u8; 32] = bytes[0..32].try_into().map_err(|_| {
            ContractSBEDecodeError::ContractSBEContractIdBytesConversionError
        })?;

        let registry_index = u64::from_le_bytes(
            bytes[32..40]
                .try_into()
                .map_err(|_| ContractSBEDecodeError::ContractSBERegistryIndexBytesConversionError)?,
        );

        Ok(Contract::new(contract_id, registry_index))
    }
}
