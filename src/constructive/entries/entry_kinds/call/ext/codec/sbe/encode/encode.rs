use crate::constructive::calldata::element::sbe::encode::list::encode_calldata_elements_sbe;
use crate::constructive::entry::entry_kinds::call::call::Call;
use crate::constructive::entry::entry_kinds::call::ext::codec::sbe::encode::error::encode_error::CallSBEEncodeError;

type Bytes = Vec<u8>;

const CALL_ENTRY_KIND_BYTE: u8 = 0x01;

fn push_length_prefixed(
    out: &mut Bytes,
    payload: &[u8],
    too_large: fn(usize) -> CallSBEEncodeError,
) -> Result<(), CallSBEEncodeError> {
    let len_u32 = u32::try_from(payload.len()).map_err(|_| too_large(payload.len()))?;
    out.extend_from_slice(&len_u32.to_le_bytes());
    out.extend_from_slice(payload);
    Ok(())
}

impl Call {
    /// Structural Byte-scope Encoding (SBE) for `Call`.
    ///
    /// Layout: `0x01` tag, length-prefixed `RootAccount` SBE, length-prefixed `Contract` SBE (40),
    /// `MethodIndex` SBE (2 bytes), length-prefixed calldata list SBE, `OpsBudget` SBE (5),
    /// `OpsPrice` SBE (8), `Target` SBE (8).
    pub fn encode_sbe(&self) -> Result<Bytes, CallSBEEncodeError> {
        let account_bytes = self.account.encode_sbe();

        let contract_bytes = self.contract.encode_sbe();

        let calldata_bytes = encode_calldata_elements_sbe(&self.calldata_elements);

        let mut bytes = Bytes::new();
        bytes.push(CALL_ENTRY_KIND_BYTE);

        push_length_prefixed(&mut bytes, &account_bytes, |len| {
            CallSBEEncodeError::CallSBERootAccountPayloadTooLargeForU32LengthPrefix { len }
        })?;

        push_length_prefixed(&mut bytes, &contract_bytes, |len| {
            CallSBEEncodeError::CallSBEContractPayloadTooLargeForU32LengthPrefix { len }
        })?;

        bytes.extend_from_slice(&self.method_index.encode_sbe());

        push_length_prefixed(&mut bytes, &calldata_bytes, |len| {
            CallSBEEncodeError::CallSBECalldataPayloadTooLargeForU32LengthPrefix { len }
        })?;

        bytes.extend_from_slice(&self.ops_budget.encode_sbe());
        bytes.extend_from_slice(&self.ops_price.encode_sbe());
        bytes.extend_from_slice(&self.target.encode_sbe());

        Ok(bytes)
    }
}
