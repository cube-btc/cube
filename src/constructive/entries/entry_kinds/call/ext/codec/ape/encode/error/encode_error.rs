use crate::constructive::calldata::calldata_elements::ape::encode::error::encode_error::CalldataElementAPEEncodeError;
use crate::constructive::core_types::method_index::ext::codec::ape::encode::error::encode_error::MethodIndexAPEEncodeError;
use crate::constructive::core_types::ops_price::ext::codec::ape::encode::error::encode_error::OpsPriceAPEEncodeError;
use crate::constructive::core_types::target::ext::codec::ape::encode::error::encode_error::TargetAPEEncodeError;
use crate::constructive::entity::account::root_account::ext::codec::ape::encode::error::encode_error::RootAccountAPEEncodeError;
use crate::constructive::entity::contract::ape::encode::error::encode_error::ContractAPEEncodeError;

/// Airly Payload Encoding (APE) encoding error for `Call`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallAPEEncodeError {
    AccountAPEEncodeError(RootAccountAPEEncodeError),
    ContractAPEEncodeError(ContractAPEEncodeError),
    UnableToRetrieveContractMethodsLenFromRegistery([u8; 32]),
    MethodIndexAPEEncodeError(MethodIndexAPEEncodeError),
    CalldataElementAPEEncodeError(CalldataElementAPEEncodeError),
    OpsPriceAPEEncodeError(OpsPriceAPEEncodeError),
    TargetAPEEncodeError(TargetAPEEncodeError),
}
