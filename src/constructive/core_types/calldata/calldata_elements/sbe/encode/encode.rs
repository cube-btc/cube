use crate::constructive::calldata::element::calldata_element::CalldataElement;
use crate::constructive::calldata::element::sbe::encode::error::encode_error::CalldataElementSBEEncodeError;

type Bytes = Vec<u8>;

impl CalldataElement {
    /// Structural Byte-scope Encoding (SBE) for a single `CalldataElement`.
    ///
    /// Layout: element-type bytecode (see [`CalldataElementType::bytecode`]), then a
    /// variant-specific payload (fixed-size scalars, length-prefixed `Varbytes`, or nested
    /// `Account`/`Contract` SBE).
    pub fn encode_sbe(&self) -> Result<Bytes, CalldataElementSBEEncodeError> {
        // 1 Validate payload invariants.
        self.validate()
            .map_err(CalldataElementSBEEncodeError::ValidationError)?;

        // 2 Write type tag and variant payload.
        let mut bytes = self.element_type().bytecode();

        match self {
            CalldataElement::U8(value) => bytes.push(*value),
            CalldataElement::U16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            CalldataElement::U32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            CalldataElement::U64(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            CalldataElement::Bool(value) => bytes.push(u8::from(*value)),
            CalldataElement::Account(account) => bytes.extend(account.encode_sbe()),
            CalldataElement::Contract(contract) => bytes.extend(contract.encode_sbe()),
            CalldataElement::Bytes(data) => bytes.extend_from_slice(data),
            CalldataElement::Varbytes(data) => {
                bytes.extend_from_slice(&(data.len() as u16).to_le_bytes());
                bytes.extend_from_slice(data);
            }
            CalldataElement::Payable(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }

        // 3 Return bytes.
        Ok(bytes)
    }
}
