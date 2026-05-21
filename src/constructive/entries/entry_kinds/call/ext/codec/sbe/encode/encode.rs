use crate::constructive::calldata::element::sbe::encode::list::encode_calldata_elements_sbe;
use crate::constructive::entry::entry_kinds::call::call::Call;
use crate::constructive::entry::entry_kinds::call::ext::codec::sbe::encode::error::encode_error::CallSBEEncodeError;

type Bytes = Vec<u8>;

const CALL_ENTRY_KIND_BYTE: u8 = 0x01;

impl Call {
    /// Structural Byte-scope Encoding (SBE) for `Call`.
    ///
    /// Layout: `0x01` tag, length-prefixed `RootAccount` SBE, length-prefixed `Contract` SBE (40),
    /// `MethodIndex` SBE (2 bytes), length-prefixed calldata list SBE, variable-length `OpsBudget` SBE,
    /// `OpsPrice` SBE (8), `Target` SBE (8).
    pub fn encode_sbe(&self) -> Result<Bytes, CallSBEEncodeError> {
        // 1 Encode nested payloads.
        let account_bytes = self.account.encode_sbe();
        let contract_bytes = self.contract.encode_sbe();
        let calldata_bytes = encode_calldata_elements_sbe(&self.calldata_elements)
            .map_err(CallSBEEncodeError::CalldataElementSBEEncodeError)?;

        // 2 Ensure payload lengths fit `u32` length prefixes.
        let account_len_u32 = u32::try_from(account_bytes.len()).map_err(|_| {
            CallSBEEncodeError::CallSBERootAccountPayloadTooLargeForU32LengthPrefix {
                len: account_bytes.len(),
            }
        })?;
        let contract_len_u32 = u32::try_from(contract_bytes.len()).map_err(|_| {
            CallSBEEncodeError::CallSBEContractPayloadTooLargeForU32LengthPrefix {
                len: contract_bytes.len(),
            }
        })?;
        let calldata_len_u32 = u32::try_from(calldata_bytes.len()).map_err(|_| {
            CallSBEEncodeError::CallSBECalldataPayloadTooLargeForU32LengthPrefix {
                len: calldata_bytes.len(),
            }
        })?;

        // 3 Initialize bytes and write layout.
        let mut bytes = Bytes::new();
        bytes.push(CALL_ENTRY_KIND_BYTE);
        bytes.extend_from_slice(&account_len_u32.to_le_bytes());
        bytes.extend_from_slice(&account_bytes);
        bytes.extend_from_slice(&contract_len_u32.to_le_bytes());
        bytes.extend_from_slice(&contract_bytes);
        bytes.extend_from_slice(&self.method_index.encode_sbe());
        bytes.extend_from_slice(&calldata_len_u32.to_le_bytes());
        bytes.extend_from_slice(&calldata_bytes);
        bytes.extend_from_slice(&self.ops_budget.encode_sbe());
        bytes.extend_from_slice(&self.ops_price.encode_sbe());
        bytes.extend_from_slice(&self.target.encode_sbe());

        // 4 Return bytes.
        Ok(bytes)
    }
}
