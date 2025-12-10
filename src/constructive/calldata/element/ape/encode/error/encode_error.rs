use crate::constructive::entity::account::ape::encode::error::encode_error::AccountAPEEncodeError;
use crate::constructive::entity::contract::ape::encode::error::encode_error::ContractAPEEncodeError;

/// Enum to represent errors that can occur when encoding a `CallElement` as an Airly Payload Encoding (APE) bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalldataElementAPEEncodeError {
    AccountAPEEncodeError(AccountAPEEncodeError),
    ContractAPEEncodeError(ContractAPEEncodeError),
}
