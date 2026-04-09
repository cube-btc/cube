use crate::constructive::entry::entry_fields::entities::account::account::ape::encode::error::encode_error::AccountAPEEncodeError;
use crate::constructive::entry::entry_fields::entities::contract::ape::encode::error::encode_error::ContractAPEEncodeError;

/// Enum to represent errors that can occur when encoding a `CallElement` as an Airly Payload Encoding (APE) bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalldataElementAPEEncodeError {
    AccountAPEEncodeError(AccountAPEEncodeError),
    ContractAPEEncodeError(ContractAPEEncodeError),
}
