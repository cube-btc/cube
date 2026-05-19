use crate::constructive::core_types::method_index::ext::codec::sbe::decode::error::decode_error::MethodIndexSBEDecodeError;
use crate::constructive::core_types::ops_budget::ext::codec::sbe::decode::error::decode_error::OpsBudgetSBEDecodeError;
use crate::constructive::core_types::ops_price::ext::codec::sbe::decode::error::decode_error::OpsPriceSBEDecodeError;
use crate::constructive::core_types::target::ext::codec::sbe::decode::error::decode_error::TargetSBEDecodeError;
use crate::constructive::entity::account::root_account::ext::codec::sbe::decode::error::decode_error::RootAccountSBEDecodeError;
use crate::constructive::calldata::element::sbe::decode::error::decode_errors::CalldataElementsSBEDecodeError;
use crate::constructive::entity::contract::ext::codec::sbe::decode::error::decode_error::ContractSBEDecodeError;

/// Errors that can occur when decoding a `Call` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallSBEDecodeError {
    InvalidEntryKindByteError { expected: u8, got: u8 },
    CallSBEInsufficientBytesForRootAccountLengthPrefix { got_total: usize },
    CallSBELengthPrefixBytesConversionError,
    CallSBERootAccountLengthPrefixExceedsPayload {
        root_len: usize,
        got_after_prefix: usize,
    },
    CallSBERootAccount(RootAccountSBEDecodeError),
    CallSBEInsufficientBytesForContractLengthPrefix { got_total: usize },
    CallSBEContractLengthPrefixExceedsPayload {
        contract_len: usize,
        got_after_prefix: usize,
    },
    CallSBEContract(ContractSBEDecodeError),
    CallSBEInsufficientBytesForMethodIndex { got_total: usize },
    CallSBEMethodIndex(MethodIndexSBEDecodeError),
    CallSBEInsufficientBytesForCalldataLengthPrefix { got_total: usize },
    CallSBECalldataLengthPrefixExceedsPayload {
        calldata_len: usize,
        got_after_prefix: usize,
    },
    CallSBECalldata(CalldataElementsSBEDecodeError),
    CallSBEInsufficientBytesForOpsBudget { got_total: usize },
    CallSBEOpsBudget(OpsBudgetSBEDecodeError),
    CallSBEInsufficientBytesForOpsPrice { got_total: usize },
    CallSBEOpsPrice(OpsPriceSBEDecodeError),
    CallSBEInsufficientBytesForTarget { got_total: usize },
    CallSBETarget(TargetSBEDecodeError),
    CallSBETrailingBytesAfterCall { trailing: usize },
}
