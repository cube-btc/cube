use crate::constructive::valtype::val::atomic_val::cpe::encode::encode_error::AtomicValCPEEncodeError;

/// Types for account key.
type ExpectedAccountKey = [u8; 32];
type FoundAccountKey = [u8; 32];

/// Types for ops price.
type ExpectedBaseOpsPrice = u32;
type FoundBaseOpsPrice = u32;

/// The error type for encoding a call as a CPE.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallCPEEncodeError {
    AccountKeyMismatch(ExpectedAccountKey, FoundAccountKey),
    ContractRankNotFoundAtContractId([u8; 32]),
    ContractBodyNotFoundAtContractId([u8; 32]),
    ContractMethodCountNotFoundAtContractId([u8; 32]),
    MethodIndexCPEEncodeError(AtomicValCPEEncodeError),
    BaseOpsPriceMismatch(ExpectedBaseOpsPrice, FoundBaseOpsPrice),
}
