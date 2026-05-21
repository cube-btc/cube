use crate::constructive::calldata::element::calldata_element::CalldataElement;

type Bytes = Vec<u8>;

impl CalldataElement {
    /// Structural Byte-scope Encoding (SBE) for a single `CalldataElement`.
    pub fn encode_sbe(&self) -> Bytes {
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
                let len = data.len() as u16;
                bytes.extend_from_slice(&len.to_le_bytes());
                bytes.extend_from_slice(data);
            }
            CalldataElement::Payable(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }

        bytes
    }
}
