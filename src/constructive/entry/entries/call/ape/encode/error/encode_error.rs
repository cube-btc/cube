use crate::constructive::calldata::element::ape::encode::error::encode_error::CalldataElementAPEEncodeError;
use crate::constructive::entity::account::ape::encode::error::encode_error::AccountAPEEncodeError;
use crate::constructive::entity::contract::ape::encode::error::encode_error::ContractAPEEncodeError;
use crate::constructive::valtype::val::atomic_val::ape::encode::error::encode_error::AtomicValAPEEncodeError;

/// Types for ops price.
type ExpectedBaseOpsPrice = u32;
type FoundBaseOpsPrice = u32;

/// Airly Payload Encoding (APE) encoding error for `Call`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallAPEEncodeError {
    AccountAPEEncodeError(AccountAPEEncodeError),
    ContractAPEEncodeError(ContractAPEEncodeError),
    MethodCallAPEEncodeError(AtomicValAPEEncodeError),
    CalldataElementAPEEncodeError(CalldataElementAPEEncodeError),
    BaseOpsPriceMismatch(ExpectedBaseOpsPrice, FoundBaseOpsPrice),
}
