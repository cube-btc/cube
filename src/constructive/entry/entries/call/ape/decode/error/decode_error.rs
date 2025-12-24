use crate::constructive::calldata::element::ape::decode::error::decode_errors::CalldataElementAPEDecodeError;
use crate::constructive::entity::account::root_account::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::entity::contract::ape::decode::error::decode_error::ContractAPEDecodeError;
use crate::constructive::valtype::val::atomic_val::ape::decode::error::decode_error::AtomicValAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Airly Payload Encoding (APE) decoding error for `Call`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallEntryAPEDecodeError {
    AccountAPEDecodeError(RootAccountAPEDecodeError),
    ContractAPEDecodeError(ContractAPEDecodeError),
    MethodIndexAPEDecodeError(AtomicValAPEDecodeError),
    CalldataElementAPEDecodeError(CalldataElementAPEDecodeError),
    OpsBudgetPresentBitCollectError,
    OpsBudgetAPEDecodeError(ShortValAPEDecodeError),
    OpsPriceOverheadPresentBitCollectError,
    OpsPriceOverheadAPEDecodeError(ShortValAPEDecodeError),
}
