use crate::constructive::calldata::element::ape::encode::error::encode_error::CalldataElementAPEEncodeError;
use crate::constructive::entity::account::root_account::ape::encode::error::encode_error::RootAccountAPEEncodeError;
use crate::constructive::entity::contract::ape::encode::error::encode_error::ContractAPEEncodeError;
use crate::constructive::valtype::val::atomic_val::ape::encode::error::encode_error::AtomicValAPEEncodeError;

/// Types for ops price.
type ExpectedBaseOpsPrice = u32;
type FoundBaseOpsPrice = u32;

/// Airly Payload Encoding (APE) encoding error for `Call`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallAPEEncodeError {
    AccountAPEEncodeError(RootAccountAPEEncodeError),
    ContractAPEEncodeError(ContractAPEEncodeError),
    MethodCallAPEEncodeError(AtomicValAPEEncodeError),
    CalldataElementAPEEncodeError(CalldataElementAPEEncodeError),
    BaseOpsPriceMismatch(ExpectedBaseOpsPrice, FoundBaseOpsPrice),
}
