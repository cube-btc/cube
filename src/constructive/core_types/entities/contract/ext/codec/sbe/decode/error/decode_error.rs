/// Errors that can occur when decoding a `Contract` from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContractSBEDecodeError {
    ContractSBEInvalidPayloadLength { got: usize, expected: usize },
    ContractSBEContractIdBytesConversionError,
    ContractSBERegistryIndexBytesConversionError,
}
