use crate::constructive::calldata::element::calldata_element::CalldataElement;

type Bytes = Vec<u8>;

/// Encodes a calldata element list as SBE bytes.
///
/// Layout: `u32` little-endian element count, then each element's [`CalldataElement::encode_sbe`] payload.
pub fn encode_calldata_elements_sbe(elements: &[CalldataElement]) -> Bytes {
    let mut bytes = Bytes::new();
    bytes.extend_from_slice(&(elements.len() as u32).to_le_bytes());
    for element in elements {
        bytes.extend(element.encode_sbe());
    }
    bytes
}
