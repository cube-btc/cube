use crate::constructive::calldata::element::sbe::decode::list::decode_calldata_elements_sbe;
use crate::constructive::core_types::method_index::method_index::MethodIndex;
use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::entry::entry_kinds::call::call::Call;
use crate::constructive::entry::entry_kinds::call::ext::codec::sbe::decode::error::decode_error::CallSBEDecodeError;

const CALL_ENTRY_KIND_BYTE: u8 = 0x01;

const METHOD_INDEX_SBE_LEN: usize = 2;
const OPS_BUDGET_SBE_LEN: usize = 5;
const OPS_PRICE_SBE_LEN: usize = 8;
const TARGET_SBE_LEN: usize = 8;

fn read_u32_len_prefix(
    bytes: &[u8],
    insufficient: impl FnOnce(usize) -> CallSBEDecodeError,
) -> Result<(u32, &[u8]), CallSBEDecodeError> {
    if bytes.len() < 4 {
        return Err(insufficient(bytes.len()));
    }
    let len = u32::from_le_bytes(bytes[0..4].try_into().map_err(|_| {
        CallSBEDecodeError::CallSBELengthPrefixBytesConversionError
    })?);
    Ok((len, &bytes[4..]))
}

fn take_length_prefixed<'a>(
    bytes: &'a [u8],
    insufficient_prefix: impl FnOnce(usize) -> CallSBEDecodeError,
    prefix_exceeds: impl FnOnce(usize, usize) -> CallSBEDecodeError,
) -> Result<(&'a [u8], &'a [u8]), CallSBEDecodeError> {
    let (len_u32, rest) = read_u32_len_prefix(bytes, insufficient_prefix)?;
    let len = len_u32 as usize;
    if rest.len() < len {
        return Err(prefix_exceeds(len, rest.len()));
    }
    let (payload, after) = rest.split_at(len);
    Ok((payload, after))
}

impl Call {
    /// Decodes a `Call` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Call, CallSBEDecodeError> {
        if bytes.is_empty() {
            return Err(CallSBEDecodeError::CallSBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: 0,
            });
        }
        if bytes[0] != CALL_ENTRY_KIND_BYTE {
            return Err(CallSBEDecodeError::InvalidEntryKindByteError {
                expected: CALL_ENTRY_KIND_BYTE,
                got: bytes[0],
            });
        }

        let (account_slice, rest) = take_length_prefixed(
            &bytes[1..],
            |got_total| CallSBEDecodeError::CallSBEInsufficientBytesForRootAccountLengthPrefix {
                got_total: got_total + 1,
            },
            |root_len, got_after_prefix| {
                CallSBEDecodeError::CallSBERootAccountLengthPrefixExceedsPayload {
                    root_len,
                    got_after_prefix: got_after_prefix + 1,
                }
            },
        )?;
        let account = RootAccount::decode_sbe(account_slice)
            .map_err(CallSBEDecodeError::CallSBERootAccount)?;

        let (contract_slice, rest) = take_length_prefixed(
            rest,
            |got_total| CallSBEDecodeError::CallSBEInsufficientBytesForContractLengthPrefix { got_total },
            |contract_len, got_after_prefix| {
                CallSBEDecodeError::CallSBEContractLengthPrefixExceedsPayload {
                    contract_len,
                    got_after_prefix,
                }
            },
        )?;
        let contract = Contract::decode_sbe(contract_slice)
            .map_err(CallSBEDecodeError::CallSBEContract)?;

        if rest.len() < METHOD_INDEX_SBE_LEN {
            return Err(CallSBEDecodeError::CallSBEInsufficientBytesForMethodIndex {
                got_total: bytes.len(),
            });
        }
        let method_index = MethodIndex::decode_sbe(&rest[0..METHOD_INDEX_SBE_LEN])
            .map_err(CallSBEDecodeError::CallSBEMethodIndex)?;
        let rest = &rest[METHOD_INDEX_SBE_LEN..];

        let (calldata_slice, rest) = take_length_prefixed(
            rest,
            |got_total| CallSBEDecodeError::CallSBEInsufficientBytesForCalldataLengthPrefix { got_total },
            |calldata_len, got_after_prefix| {
                CallSBEDecodeError::CallSBECalldataLengthPrefixExceedsPayload {
                    calldata_len,
                    got_after_prefix,
                }
            },
        )?;
        let calldata_elements = decode_calldata_elements_sbe(calldata_slice)
            .map_err(CallSBEDecodeError::CallSBECalldata)?;

        if rest.len() < OPS_BUDGET_SBE_LEN {
            return Err(CallSBEDecodeError::CallSBEInsufficientBytesForOpsBudget {
                got_total: bytes.len(),
            });
        }
        let ops_budget = OpsBudget::decode_sbe(&rest[0..OPS_BUDGET_SBE_LEN])
            .map_err(CallSBEDecodeError::CallSBEOpsBudget)?;
        let rest = &rest[OPS_BUDGET_SBE_LEN..];

        if rest.len() < OPS_PRICE_SBE_LEN {
            return Err(CallSBEDecodeError::CallSBEInsufficientBytesForOpsPrice {
                got_total: bytes.len(),
            });
        }
        let ops_price = OpsPrice::decode_sbe(&rest[0..OPS_PRICE_SBE_LEN])
            .map_err(CallSBEDecodeError::CallSBEOpsPrice)?;
        let rest = &rest[OPS_PRICE_SBE_LEN..];

        if rest.len() < TARGET_SBE_LEN {
            return Err(CallSBEDecodeError::CallSBEInsufficientBytesForTarget {
                got_total: bytes.len(),
            });
        }
        let target = Target::decode_sbe(&rest[0..TARGET_SBE_LEN])
            .map_err(CallSBEDecodeError::CallSBETarget)?;
        let tail = &rest[TARGET_SBE_LEN..];

        if !tail.is_empty() {
            return Err(CallSBEDecodeError::CallSBETrailingBytesAfterCall {
                trailing: tail.len(),
            });
        }

        Ok(Call::new(
            account,
            contract,
            method_index,
            calldata_elements,
            ops_budget,
            ops_price,
            target,
        ))
    }
}
