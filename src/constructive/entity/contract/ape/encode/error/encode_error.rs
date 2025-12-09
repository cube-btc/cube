/// Contract ID.
type ContractId = [u8; 32];

/// Enum to represent errors that can occur when encoding a `Contract` as a bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContractAPEEncodeError {
    UndeployedContractCannotBeEncodedError(ContractId),
    RankNotFoundError(ContractId),
}
