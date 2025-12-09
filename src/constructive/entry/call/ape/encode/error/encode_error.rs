use crate::constructive::calldata::element::ape::encode::error::encode_error::CallElementAPEEncodeError;
use crate::constructive::valtype::val::atomic_val::ape::encode::error::encode_error::AtomicValAPEEncodeError;

/// Types for account key.
type ExpectedAccountKey = [u8; 32];
type FoundAccountKey = [u8; 32];

/// Types for ops price.
type ExpectedBaseOpsPrice = u32;
type FoundBaseOpsPrice = u32;

/// Airly Payload Encoding (APE) encoding error for `Call`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallAPEEncodeError {
    AccountKeyMismatch(ExpectedAccountKey, FoundAccountKey),
    ContractRankNotFoundAtContractId([u8; 32]),
    ContractBodyNotFoundAtContractId([u8; 32]),
    ContractMethodCountNotFoundAtContractId([u8; 32]),
    MethodIndexAPEEncodeError(AtomicValAPEEncodeError),
    CallElementAPEEncodeError(CallElementAPEEncodeError),
    BaseOpsPriceMismatch(ExpectedBaseOpsPrice, FoundBaseOpsPrice),
}
