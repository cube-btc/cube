use crate::constructive::calldata::calldata_elements::ape::decode::error::decode_errors::CalldataElementAPEDecodeError;
use crate::constructive::core_types::method_index::ext::codec::ape::decode::error::decode_error::MethodIndexAPEDecodeError;
use crate::constructive::core_types::ops_budget::ext::codec::ape::decode::error::decode_error::OpsBudgetAPEDecodeError;
use crate::constructive::core_types::ops_price::ext::codec::ape::decode::error::decode_error::OpsPriceAPEDecodeError;
use crate::constructive::core_types::target::ext::codec::ape::decode::error::decode_error::TargetAPEDecodeError;
use crate::constructive::entity::account::root_account::ext::codec::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::entity::contract::ape::decode::error::decode_error::ContractAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Airly Payload Encoding (APE) decoding error for `Call`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallEntryAPEDecodeError {
    AccountAPEDecodeError(RootAccountAPEDecodeError),
    ContractAPEDecodeError(ContractAPEDecodeError),
    UnableToRetrieveContractMethodsLenFromRegistry([u8; 32]),
    UnableToRetrieveMethodArgTypesFromRegistry {
        contract_id: [u8; 32],
        method_index: u16,
    },
    CalldataCountMismatch {
        expected: usize,
        got: usize,
    },
    CalldataCountAPEDecodeError(ShortValAPEDecodeError),
    MethodIndexAPEDecodeError(MethodIndexAPEDecodeError),
    CalldataElementAPEDecodeError(CalldataElementAPEDecodeError),
    OpsBudgetAPEDecodeError(OpsBudgetAPEDecodeError),
    OpsPriceAPEDecodeError(OpsPriceAPEDecodeError),
    TargetAPEDecodeError(TargetAPEDecodeError),
}
